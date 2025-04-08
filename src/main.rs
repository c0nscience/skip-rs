use anyhow::Context;
use askama::Template;
use rspotify::model::{AlbumId, Market};
use rspotify::prelude::BaseClient;
use rspotify::{ClientCredsSpotify, Credentials};
use sqlx::postgres::PgPoolOptions;
use tower_http::services::{ServeDir, ServeFile};

use askama_axum::IntoResponse;
use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;

use std::net::SocketAddr;
use tower_http::compression::CompressionLayer;

use tracing::info;

pub mod categories;
pub mod entries;
pub mod errors;
pub mod states;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let port = dotenvy::var("PORT").map_or_else(|_| Ok(3000), |p| p.parse::<u16>())?;
    let database_url =
        dotenvy::var("DATABASE_URL").context("no postgres connection url provided")?;

    let creds = Credentials::from_env().context("no spotify credentials found.")?;

    let spotify = ClientCredsSpotify::new(creds);
    // I guess I have to call this once ... after that the token should be refreshed
    spotify.request_token().await?;

    // we have to extract the spotify id from the given url ... the whole api seems to run on the
    // ids only
    // https://open.spotify.com/album/4IMxyQClyxn6nLxYAK1BDs?si=P6MKoKwATS2PXORAkUhPiA
    // https://open.spotify.com/playlist/1aRCVcpxtMrIwOHe28AlfG?si=11635db56f104035
    let uri = AlbumId::from_uri("spotify:album:4IMxyQClyxn6nLxYAK1BDs")?;
    let albums = spotify
        .album(
            uri,
            Some(Market::Country(rspotify::model::Country::Germany)),
        )
        .await?;
    println!("Response: {}", albums.name);

    let db = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(&database_url)
        .await
        .context("could not connect to database")?;
    sqlx::migrate!().run(&db).await?;

    let state = states::AppState { db, spotify };

    let app = Router::new()
        .route("/:category/categories", get(categories::handlers::list))
        .route(
            "/:category/categories/:category_id/entries",
            get(entries::handlers::list),
        )
        .route(
            "/:category/categories/:category_id/entries/:entry_id",
            get(entries::handlers::get_entry),
        )
        .route("/", get(index))
        .route("/health", get(health))
        .nest_service("/favicon.ico", ServeFile::new("assets/favicon.ico"))
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(CompressionLayer::new())
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

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

async fn index() -> impl IntoResponse {
    IndexTemplate {}
}

async fn health() -> (StatusCode, impl IntoResponse) {
    (StatusCode::OK, "OK")
}
