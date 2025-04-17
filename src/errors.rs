use axum::http::StatusCode;
use axum::http::header::InvalidHeaderValue;
use axum::response::{IntoResponse, Response};
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("not found")]
    NotFound,

    #[error("internal server error")]
    InternalError,

    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("{0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("{0}")]
    StrumError(#[from] strum::ParseError),

    #[error("{0}")]
    RSpotifyIdError(#[from] rspotify::model::IdError),

    #[error("{0}")]
    RSpotifyClientError(#[from] rspotify::ClientError),

    #[error("{0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("{0}")]
    HeaderError(#[from] InvalidHeaderValue),

    #[error("{0}")]
    AskamaError(#[from] askama::Error),

    #[error("{0}")]
    UuidError(#[from] sqlx::types::uuid::Error),

    #[error("{0}")]
    ToStrError(#[from] reqwest::header::ToStrError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        use AppError::{
            Anyhow, AskamaError, HeaderError, InternalError, NotFound, RSpotifyClientError,
            RSpotifyIdError, Sqlx, StrumError, ToStrError, UrlParseError, Utf8Error, UuidError,
        };

        match self {
            NotFound => (StatusCode::NOT_FOUND).into_response(),
            InternalError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            Anyhow(err) => {
                error!("anyhow: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            Sqlx(err) => {
                error!("sqlx: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            Utf8Error(err) => {
                error!("utf8: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            StrumError(err) => {
                error!("strum: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            RSpotifyIdError(err) => {
                error!("rspotify: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            RSpotifyClientError(err) => {
                error!("rspotify: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            UrlParseError(err) => {
                error!("url parser: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            HeaderError(err) => {
                error!("header error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            AskamaError(err) => {
                error!("askama error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            UuidError(err) => {
                error!("uuid error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            ToStrError(err) => {
                error!("to str error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
        }
    }
}
