use std::str::FromStr;

use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Path, State};

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
    Ok(CategoriesTemplate {
        category_type,
        categories,
    })
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
    Ok(ListTemplate { categories })
}

pub async fn admin_new() -> Result<impl IntoResponse, errors::AppError> {
    todo!();
    Ok(())
}

pub async fn admin_create() -> Result<impl IntoResponse, errors::AppError> {
    todo!();
    Ok(())
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
    Ok(EditTemplate { category })
}

pub async fn admin_update() -> Result<impl IntoResponse, errors::AppError> {
    todo!();
    Ok(())
}

pub async fn admin_delete() -> Result<impl IntoResponse, errors::AppError> {
    todo!();
    Ok(())
}
