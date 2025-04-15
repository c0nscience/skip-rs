use std::str::FromStr;

use askama::Template;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{Html, IntoResponse},
};
use axum_extra::extract::Form;
use serde::Deserialize;

use crate::{categories::CategoryType, errors, states};

use super::Category;

#[derive(Template)]
#[template(path = "categories.html")]
struct CategoriesTemplate {
    category_type: CategoryType,
    categories: Vec<Category>,
}

pub async fn list(
    Path(category): Path<String>,
    State(state): State<states::AppState>,
) -> Result<impl IntoResponse, errors::AppError> {
    let category_type = CategoryType::from_str(&category)?;
    let categories = super::list_all_by_type(&state.db, &category_type).await?;
    Ok(Html(
        CategoriesTemplate {
            category_type,
            categories,
        }
        .render()?,
    ))
}

#[derive(Template)]
#[template(path = "admin_categories.html")]
struct ListTemplate {
    categories: Vec<Category>,
}

pub async fn admin_list(
    State(state): State<states::AppState>,
) -> Result<impl IntoResponse, errors::AppError> {
    let categories = super::list_all(&state.db).await?;
    Ok(Html(ListTemplate { categories }.render()?))
}

#[derive(Template)]
#[template(path = "admin_categories_create.html")]
struct CreateTemplate {}

pub async fn admin_new() -> Result<impl IntoResponse, errors::AppError> {
    Ok(Html(CreateTemplate {}.render()?))
}

#[derive(Deserialize)]
pub struct CategoryCreateForm {
    name: String,
    image_url: String,
    category_type: String,
}

pub async fn admin_create(
    State(state): State<states::AppState>,
    Form(category_form): Form<CategoryCreateForm>,
) -> Result<impl IntoResponse, errors::AppError> {
    let category_type = CategoryType::from_str(&category_form.category_type)?;
    let id = super::create(
        &state.db,
        &category_form.name,
        &category_form.image_url,
        &category_type,
    )
    .await?;

    let mut headers = HeaderMap::new();
    let path = format!("/admin/categories/{}", id.to_string());
    headers.insert("HX-Redirect", path.parse()?);
    Ok(headers)
}

#[derive(Template)]
#[template(path = "admin_categories_edit.html")]
struct EditTemplate {
    category: Category,
}
pub async fn admin_get_category(
    Path(category_id): Path<String>,
    State(state): State<states::AppState>,
) -> Result<impl IntoResponse, errors::AppError> {
    let category = super::get(&state.db, &category_id).await?;
    Ok(Html(EditTemplate { category }.render()?))
}

#[derive(Deserialize)]
pub struct CategoryEditForm {
    id: String,
    name: String,
    image_url: String,
    category_type: String,
}

impl TryInto<Category> for CategoryEditForm {
    type Error = anyhow::Error;

    fn try_into(self) -> anyhow::Result<Category, Self::Error> {
        let id = sqlx::types::Uuid::parse_str(&self.id)?;
        Ok(Category {
            id,
            name: self.name,
            image_url: self.image_url,
            category_type: CategoryType::from_str(&self.category_type)?,
        })
    }
}

pub async fn admin_update(
    State(state): State<states::AppState>,
    Form(category_form): Form<CategoryEditForm>,
) -> Result<impl IntoResponse, errors::AppError> {
    let category = category_form.try_into()?;
    super::update(&state.db, &category).await?;
    let mut headers = HeaderMap::new();
    let path = format!("/admin/categories/{}", category.id.to_string());
    headers.insert("HX-Redirect", path.parse()?);
    Ok(headers)
}

pub async fn admin_delete(
    Path(category_id): Path<String>,
    State(state): State<states::AppState>,
) -> Result<impl IntoResponse, errors::AppError> {
    super::delete(&state.db, &category_id).await?;
    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", "/admin/categories".parse().unwrap());
    Ok(headers)
}
