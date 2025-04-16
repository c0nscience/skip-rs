use std::str::FromStr;

use anyhow::Result;
use askama::Template;
use axum::{
    Form,
    extract::{Path, State},
    http::HeaderMap,
    response::{Html, IntoResponse},
};
use rspotify::{
    ClientCredsSpotify,
    model::{AlbumId, Id, Image, PlaylistId},
    prelude::BaseClient,
};
use serde::Deserialize;
use serde_json::json;
use serde_with::NoneAsEmptyString;
use serde_with::serde_as;
use sqlx::types::Uuid;
use strum::Display;
use url::Url;

use crate::{
    MARKET,
    categories::{self, CategoryType},
    errors, states, with_height,
};

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
    let entries = super::list_all_visible_by_category(&_state.db, &category_id).await?;
    Ok(Html(
        EntriesTemplate {
            category_id,
            category_type,
            entries,
        }
        .render()?,
    ))
}

#[derive(Deserialize, Debug, PartialEq, Display)]
pub enum Room {
    Playroom,
    Bathroom,
    Kitchen,
    LivingRoom,
}

#[derive(Template)]
#[template(path = "entry.html")]
struct EntryTemplate {
    category_id: String,
    category_type: CategoryType,
    entry_id: String,
    image_url: String,
    rooms: Vec<Room>,
}

pub async fn get_entry(
    Path((category, category_id, entry_id)): Path<(String, String, String)>,
    State(state): State<states::AppState>,
) -> Result<impl IntoResponse, errors::AppError> {
    let category_type = CategoryType::from_str(&category)?;
    let entry = super::get(&state.db, &entry_id).await?;
    let rooms = state.ha_client.available_rooms().await?;
    Ok(Html(
        EntryTemplate {
            category_id,
            category_type,
            entry_id,
            image_url: entry.image_url,
            rooms,
        }
        .render()?,
    ))
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
    Ok(Html(ListTemplate { categories }.render()?))
}

struct CategoryEntryEditModel {
    id: sqlx::types::Uuid,
    name: String,
}

#[derive(Template)]
#[template(path = "admin_entries_edit.html")]
struct EditTemplate {
    entry: EntryEditModel,
    categories: Vec<CategoryEntryEditModel>,
}

pub async fn admin_get_entry(
    Path(entry_id): Path<String>,
    State(state): State<states::AppState>,
) -> Result<impl IntoResponse, errors::AppError> {
    let entry = super::get(&state.db, &entry_id).await?;
    let categories = categories::list_all(&state.db)
        .await?
        .iter()
        .map(|c| CategoryEntryEditModel {
            id: c.id.clone(),
            name: c.name.clone(),
        })
        .collect();
    Ok(Html(EditTemplate { entry, categories }.render()?))
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct EntryEditForm {
    id: String,
    name: String,
    image_url: String,
    entry_type: String,
    spotify_uri: String,
    spotify_id: String,
    play_count: i16,
    blob: serde_json::Value,

    #[serde_as(as = "NoneAsEmptyString")]
    category_id: Option<String>,
    #[serde(default)]
    visible: bool,
}

impl TryInto<EntryEditModel> for EntryEditForm {
    type Error = anyhow::Error;

    fn try_into(self) -> anyhow::Result<EntryEditModel, Self::Error> {
        let category_id = if let Some(category_id) = self.category_id {
            Some(sqlx::types::Uuid::parse_str(&category_id)?)
        } else {
            Option::None
        };

        Ok(EntryEditModel {
            id: sqlx::types::Uuid::parse_str(&self.id)?,
            name: self.name,
            image_url: self.image_url,
            entry_type: EntryType::from_str(&self.entry_type)?,
            spotify_uri: self.spotify_uri,
            spotify_id: self.spotify_id,
            play_count: self.play_count,
            blob: self.blob,
            category_id,
            visible: self.visible,
        })
    }
}
pub async fn admin_update(
    State(state): State<states::AppState>,
    Form(entry_form): Form<EntryEditForm>,
) -> Result<impl IntoResponse, errors::AppError> {
    let entry: EntryEditModel = entry_form.try_into()?;
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
    spotify_urls: String,
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
    Ok(Html(CreateTemplate {}.render()?))
}

pub async fn admin_create(
    State(state): State<states::AppState>,
    Form(create_form): Form<CreateForm>,
) -> Result<impl IntoResponse, errors::AppError> {
    let urls: &Vec<Url> = &create_form
        .spotify_urls
        .split('\n')
        .flat_map(|s| Url::parse(s))
        .collect::<Vec<_>>();

    let mut ids: Vec<Uuid> = vec![];
    for url in urls {
        let entry = create_entry_from_url(url, None, &state.spotify).await?;
        let id = super::create(&state.db, entry).await?;
        ids.push(id);
    }

    let mut headers = HeaderMap::new();
    if ids.len() == 1 {
        let id = ids.get(0).ok_or(anyhow::anyhow!("only id did not exist"))?;

        let path = format!("/admin/entries/{}", id.to_string());
        headers.insert("HX-Redirect", path.parse()?);
    } else {
        headers.insert("HX-Redirect", "/admin/entries".parse()?);
    }
    Ok(headers)
}

async fn create_entry_from_url(
    url: &Url,
    category_id: Option<sqlx::types::Uuid>,
    spotify: &ClientCredsSpotify,
) -> anyhow::Result<EntryCreateModel> {
    let segments = url
        .path_segments()
        .map(|c| c.collect::<Vec<_>>())
        .ok_or(anyhow::anyhow!("no path available"))?;

    match segments[..] {
        ["album", id] => {
            let id = AlbumId::from_id(id)?;
            let album = spotify.album(id, MARKET).await?;
            Ok(EntryCreateModel {
                name: album.name.clone(),
                image_url: find_image(album.images.clone())?,
                entry_type: EntryType::Album,
                spotify_uri: album.id.uri(),
                spotify_id: album.id.id().to_string(),
                play_count: 0,
                blob: json!(album),
                visible: false,
                category_id,
            })
        }
        ["playlist", id] => {
            let id = PlaylistId::from_id(id)?;
            let playlist = spotify.playlist(id, None, MARKET).await?;
            Ok(EntryCreateModel {
                name: playlist.name.clone(),
                image_url: find_image(playlist.images.clone())?,
                entry_type: EntryType::Playlist,
                spotify_uri: playlist.id.uri(),
                spotify_id: playlist.id.id().to_string(),
                play_count: 0,
                blob: json!(playlist),
                visible: false,
                category_id,
            })
        }
        _ => Err(anyhow::anyhow!("url type not supporeted")),
    }
}

#[derive(Template)]
#[template(path = "admin_partial_entry_list.html")]
struct PartialEntryList {
    entries: Vec<EntryListModel>,
}

pub async fn admin_create_for_category(
    Path(category_id): Path<String>,
    State(state): State<states::AppState>,
    Form(create_form): Form<CreateForm>,
) -> Result<impl IntoResponse, errors::AppError> {
    let category_id = sqlx::types::Uuid::parse_str(&category_id)?;
    let urls: &Vec<Url> = &create_form
        .spotify_urls
        .split('\n')
        .flat_map(|s| Url::parse(s))
        .collect::<Vec<_>>();

    let mut ids: Vec<Uuid> = vec![];
    for url in urls {
        let entry = create_entry_from_url(url, Some(category_id), &state.spotify).await?;
        let id = super::create(&state.db, entry).await?;
        ids.push(id);
    }

    let entries = super::list_all_by_category(&state.db, &category_id.to_string()).await?;
    Ok(Html(PartialEntryList { entries }.render()?))
}
