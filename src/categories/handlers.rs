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
    let categories = super::list_all(&state.db, &category_type).await?;
    Ok(CategoriesTemplate {
        category_type,
        categories,
    })
}
