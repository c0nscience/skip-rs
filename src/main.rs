use anyhow::Context;
use askama::Template;
// use sqlx::postgres::PgPoolOptions;
use tower_http::services::{ServeDir, ServeFile};

use askama_axum::IntoResponse;
use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;

use std::net::SocketAddr;
use tower_http::compression::CompressionLayer;

use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let port = dotenvy::var("PORT").map_or_else(|_| Ok(3000), |p| p.parse::<u16>())?;

    // let db = PgPoolOptions::new()
    //     .max_connections(20)
    //     .acquire_timeout(std::time::Duration::from_secs(3))
    //     .connect(&database_url)
    //     .await
    //     .context("could not connect to database")?;
    // sqlx::migrate!().run(&db).await?;

    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .layer(CompressionLayer::new())
        .nest_service("/favicon.ico", ServeFile::new("assets/favicon.ico"))
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(CompressionLayer::new());
    // .with_state(state.clone());

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
