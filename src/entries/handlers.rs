use std::str::FromStr;

use anyhow::Result;
use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    Form,
    extract::{Path, State},
    http::HeaderMap,
    routing::any,
};
use rspotify::{
    model::{AlbumId, Id, Image, PlaylistId},
    prelude::BaseClient,
};
use serde::Deserialize;
use serde_json::json;
use url::Url;

use crate::{MARKET, categories::CategoryType, errors, states, with_height};

use super::{CategoryListModel, EntryCreateModel, EntryEditModel, EntryListModel, EntryType};

#[derive(Template)]
#[template(path = "entries.html")]
struct EntriesTemplate {
    category_id: String,
    category_type: CategoryType,
    entries: Vec<EntryListModel>,
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
    spotify_uri: String,
    spotify_id: String,
    play_count: i16,
    blob: serde_json::Value,
}

impl TryInto<EntryEditModel> for EntryEditForm {
    type Error = anyhow::Error;

    fn try_into(self) -> anyhow::Result<EntryEditModel, Self::Error> {
        Ok(EntryEditModel {
            id: sqlx::types::Uuid::parse_str(&self.id)?,
            name: self.name,
            image_url: self.image_url,
            entry_type: EntryType::from_str(&self.entry_type)?,
            spotify_uri: self.spotify_uri,
            spotify_id: self.spotify_id,
            play_count: self.play_count,
            blob: self.blob,
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

#[derive(Deserialize, Debug)]
pub struct CreateForm {
    spotify_url: String,
}

fn find_image(images: Vec<Image>) -> Result<String> {
    images
        .into_iter()
        .find(with_height)
        .map(|i| i.url.clone())
        .ok_or(anyhow::anyhow!("could not extract image url"))
}

#[derive(Template)]
#[template(path = "admin_entries_create.html")]
struct CreateTemplate {}

pub async fn admin_new() -> Result<impl IntoResponse, errors::AppError> {
    Ok(CreateTemplate {})
}

pub async fn admin_create(
    State(state): State<states::AppState>,
    Form(create_form): Form<CreateForm>,
) -> Result<impl IntoResponse, errors::AppError> {
    let url = Url::parse(&create_form.spotify_url)?;
    let segments = url
        .path_segments()
        .map(|c| c.collect::<Vec<_>>())
        .ok_or(anyhow::anyhow!("no path available"))?;

    let maybe_entry: anyhow::Result<EntryCreateModel> = match segments[..] {
        ["album", id] => {
            let id = AlbumId::from_id(id)?;
            let album = state.spotify.album(id, MARKET).await?;
            Ok(EntryCreateModel {
                name: album.name.clone(),
                image_url: find_image(album.images.clone())?,
                entry_type: EntryType::Album,
                spotify_uri: album.id.uri(),
                spotify_id: album.id.id().to_string(),
                play_count: 0,
                blob: json!(album),
            })
        }
        ["playlist", id] => {
            let id = PlaylistId::from_id(id)?;
            let playlist = state.spotify.playlist(id, None, MARKET).await?;
            Ok(EntryCreateModel {
                name: playlist.name.clone(),
                image_url: find_image(playlist.images.clone())?,
                entry_type: EntryType::Playlist,
                spotify_uri: playlist.id.uri(),
                spotify_id: playlist.id.id().to_string(),
                play_count: 0,
                blob: json!(playlist),
            })
        }
        _ => Err(anyhow::anyhow!("url type not supporeted")),
    };

    if let Ok(entry) = maybe_entry {
        let id = super::create(&state.db, entry).await?;
        let mut headers = HeaderMap::new();
        let path = format!("/admin/entries/{}", id.to_string());
        headers.insert("HX-Redirect", path.parse()?);
        return Ok(headers);
    }

    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", "/admin/entries".parse()?);
    Ok(headers)
}
