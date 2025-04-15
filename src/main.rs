use anyhow::Context;
use askama::Template;
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Redirect};
use rspotify::model::{AlbumId, Image, Market, PlaylistId};
use rspotify::prelude::BaseClient;
use rspotify::{ClientCredsSpotify, Credentials};
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;

use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Form, Router};
use tower_http::services::{ServeDir, ServeFile};
use url::Url;

use std::net::SocketAddr;

use tracing::info;

pub mod categories;
pub mod entries;
pub mod errors;
pub mod ha;
pub mod states;

// TODO:
// - only show categories with at least one visible entry
// - add 'visible' field to entries and categories
// - only show entries and categories which are visible
// - increase played counter on entries once an entry was started
// - add search to kids side: it should filter enrties and show a suitable list of entries
// - show play count in admin side
// - show visibility status of entries and categories in admin view
// - add ability to directly add an entry to a category in the category edit view
// - add the novel feature again that the kids side automatically updates it self once an entry is added
//  - maybe filter it on the 'client' side if only the current view is effected?
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let port = dotenvy::var("PORT").map_or_else(|_| Ok(3000), |p| p.parse::<u16>())?;
    let database_url =
        dotenvy::var("DATABASE_URL").context("no postgres connection url provided")?;

    let ha_host = dotenvy::var("HA_HOST").context("no home assistent connection url provided")?;
    let ha_token = dotenvy::var("HA_TOKEN").context("no home assistent token provided")?;

    let creds = Credentials::from_env().context("no spotify credentials found.")?;

    let spotify = ClientCredsSpotify::new(creds);
    // I guess I have to call this once ... after that the token should be refreshed
    spotify.request_token().await?;

    let db = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(&database_url)
        .await
        .context("could not connect to database")?;
    sqlx::migrate!().run(&db).await?;

    let ha_client = ha::Client::new(&ha_host, &ha_token);

    let state = states::AppState {
        db,
        spotify,
        ha_client,
    };

    let app = Router::new()
        .route("/{category}/categories", get(categories::handlers::list))
        .route(
            "/{category}/categories/{category_id}/entries",
            get(entries::handlers::list),
        )
        .route(
            "/{category}/categories/{category_id}/entries/{entry_id}",
            get(entries::handlers::get_entry).post(play),
        )
        .route("/admin", get(admin_index))
        .route("/admin/categories", get(categories::handlers::admin_list))
        .route(
            "/admin/categories/new",
            get(categories::handlers::admin_new).post(categories::handlers::admin_create),
        )
        .route(
            "/admin/categories/{category_id}",
            get(categories::handlers::admin_get_category)
                .put(categories::handlers::admin_update)
                .delete(categories::handlers::admin_delete),
        )
        .route("/admin/entries", get(entries::handlers::admin_list))
        .route(
            "/admin/entries/new",
            get(entries::handlers::admin_new).post(entries::handlers::admin_create),
        )
        .route(
            "/admin/entries/{entry_id}",
            get(entries::handlers::admin_get_entry)
                .put(entries::handlers::admin_update)
                .delete(entries::handlers::admin_delete),
        )
        .route("/admin/image-selection", post(admin_image_selection))
        .route(
            "/",
            get(|| async { Redirect::permanent("/audiobook/categories") }),
        )
        .route("/health", get(health))
        .nest_service("/favicon.ico", ServeFile::new("public/icons/favicon.ico"))
        .nest_service("/public", ServeDir::new("public"))
        .with_state(state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    if let Ok(addr) = listener.local_addr() {
        info!("server started at {}", addr);
    }

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .context("failed to start server")
}

async fn health() -> (StatusCode, impl IntoResponse) {
    (StatusCode::OK, "OK")
}

#[derive(Template)]
#[template(path = "admin.html")]
struct AdminIndexTemplate {
    category_count: u16,
    entry_count: u16,
    play_count: u16,
}

async fn admin_index() -> Result<impl IntoResponse, errors::AppError> {
    Ok(Html(
        AdminIndexTemplate {
            category_count: 42,
            entry_count: 21,
            play_count: 9000,
        }
        .render()?,
    ))
}

#[derive(Deserialize, Debug)]
pub struct ImageSelectionForm {
    spotify_url: String,
}

#[derive(Template)]
#[template(path = "admin_categories_image_selection.html")]
struct ImageSelectionTemplate {
    urls: Vec<String>,
}

pub const MARKET: Option<Market> = Some(Market::Country(rspotify::model::Country::Germany));
fn with_height(i: &Image) -> bool {
    i.height.is_some_and(|h| h == 320 || h == 300)
}

pub async fn admin_image_selection(
    State(state): State<states::AppState>,
    Form(image_selection_form): Form<ImageSelectionForm>,
) -> Result<impl IntoResponse, errors::AppError> {
    let url = Url::parse(&image_selection_form.spotify_url)?;
    let segments = url
        .path_segments()
        .map(|c| c.collect::<Vec<_>>())
        .ok_or(anyhow::anyhow!("no path available"))?;

    let image_urls: Vec<String> = match segments[..] {
        ["album", id] => {
            let id = AlbumId::from_id(id)?;
            let album = state.spotify.album(id, MARKET).await?;
            let artist_ids = album
                .artists
                .iter()
                .flat_map(|a| a.id.clone())
                .collect::<Vec<_>>();

            let mut images = state
                .spotify
                .artists(artist_ids)
                .await?
                .iter()
                .flat_map(|a| a.images.clone().into_iter().find(with_height))
                .collect::<Vec<_>>();

            if let Some(album_image) = album.images.clone().into_iter().find(with_height) {
                images.push(album_image);
            }
            images.iter().map(|i| i.url.clone()).collect()
        }
        ["playlist", id] => {
            let id = PlaylistId::from_id(id)?;
            let playlist = state.spotify.playlist(id, None, MARKET).await?;
            playlist.images.clone().into_iter().find(with_height);

            let mut images: Vec<Image> = vec![];
            if let Some(playlist_image) = playlist.images.into_iter().find(with_height) {
                images.push(playlist_image);
            }
            images.iter().map(|i| i.url.clone()).collect()
        }
        _ => vec![],
    };
    Ok(Html(ImageSelectionTemplate { urls: image_urls }.render()?))
}

#[derive(Deserialize, Debug)]
pub struct RoomSelectionForm {
    room: entries::handlers::Room,
}
pub async fn play(
    Path((category, category_id, entry_id)): Path<(String, String, String)>,
    State(state): State<states::AppState>,
    Form(room_selection_form): Form<RoomSelectionForm>,
) -> Result<impl IntoResponse, errors::AppError> {
    let entry = entries::get(&state.db, &entry_id).await?;
    state
        .ha_client
        .play(&room_selection_form.room, &entry.spotify_uri)
        .await?;

    info!("started {} in {}", &entry.name, &room_selection_form.room);
    let mut headers = HeaderMap::new();
    let path = format!("/{}/categories/{}/entries", category, category_id);
    headers.insert("HX-Redirect", path.parse()?);
    Ok(headers)
}
