use std::str::FromStr;

use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Path, State};

use crate::{categories::CategoryType, errors, states};

use super::Entry;

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
    let entries = super::list_all(&_state.db, &category_id).await?;
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
