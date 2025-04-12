use std::str::FromStr;

use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    Form,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::Deserialize;

use crate::{categories::CategoryType, errors, states};

use super::{CategoryListModel, Entry, EntryEditModel, EntryType};

#[derive(Template)]
#[template(path = "entries.html")]
struct EntriesTemplate {
    category_id: String,
    category_type: CategoryType,
    entries: Vec<Entry>,
}

pub async fn list(
    Path((category, category_id)): Path<(String, String)>,
    State(_state): State<states::AppState>,
) -> Result<impl IntoResponse, errors::AppError> {
    let category_type = CategoryType::from_str(&category)?;
    let entries = super::list_all_by_type(&_state.db, &category_id).await?;
    Ok(EntriesTemplate {
        category_id,
        category_type,
        entries,
    })
}

#[derive(Template)]
#[template(path = "entry.html")]
struct EntryTemplate {
    category_id: String,
    category_type: CategoryType,
    entry_id: String,
}

pub async fn get_entry(
    Path((category, category_id, entry_id)): Path<(String, String, String)>,
    State(_state): State<states::AppState>,
) -> Result<impl IntoResponse, errors::AppError> {
    let category_type = CategoryType::from_str(&category)?;
    Ok(EntryTemplate {
        category_id,
        category_type,
        entry_id,
    })
}

#[derive(Template)]
#[template(path = "admin_entries.html")]
struct ListTemplate {
    categories: Vec<CategoryListModel>,
}

pub async fn admin_list(
    State(state): State<states::AppState>,
) -> Result<impl IntoResponse, errors::AppError> {
    let categories = super::list_all(&state.db).await?;
    Ok(ListTemplate { categories })
}

#[derive(Template)]
#[template(path = "admin_entries_edit.html")]
struct EditTemplate {
    entry: EntryEditModel,
}

pub async fn admin_get_entry(
    Path(entry_id): Path<String>,
    State(state): State<states::AppState>,
) -> Result<impl IntoResponse, errors::AppError> {
    let entry = super::get(&state.db, &entry_id).await?;
    Ok(EditTemplate { entry })
}

#[derive(Deserialize)]
pub struct EntryEditForm {
    id: String,
    name: String,
    image_url: String,
    entry_type: String,
}

impl TryInto<EntryEditModel> for EntryEditForm {
    type Error = anyhow::Error;

    fn try_into(self) -> anyhow::Result<EntryEditModel, Self::Error> {
        Ok(EntryEditModel {
            id: sqlx::types::Uuid::parse_str(&self.id)?,
            name: self.name,
            image_url: self.image_url,
            entry_type: EntryType::from_str(&self.entry_type)?,
        })
    }
}
pub async fn admin_update(
    State(state): State<states::AppState>,
    Form(entry_form): Form<EntryEditForm>,
) -> Result<impl IntoResponse, errors::AppError> {
    let entry = entry_form.try_into()?;
    super::update(&state.db, &entry).await?;
    let mut headers = HeaderMap::new();
    let path = format!("/admin/entries/{}", entry.id.to_string());
    headers.insert("HX-Redirect", path.parse()?);
    Ok(headers)
}

pub async fn admin_delete(
    Path(entry_id): Path<String>,
    State(state): State<states::AppState>,
) -> Result<impl IntoResponse, errors::AppError> {
    super::delete(&state.db, &entry_id).await?;
    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", "/admin/entries".parse().unwrap());
    Ok(headers)
}

pub async fn admin_new() -> Result<impl IntoResponse, errors::AppError> {
    todo!();
    Ok(())
}

pub async fn admin_create() -> Result<impl IntoResponse, errors::AppError> {
    todo!();
    Ok(())
}
