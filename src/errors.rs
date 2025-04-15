use axum::http::StatusCode;
use axum::http::header::InvalidHeaderValue;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::multipart;
use thiserror::Error;
use tracing::warn;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("not found")]
    NotFound,

    #[error("internal server error")]
    InternalError,

    #[error("{0}")]
    MultipartError(#[from] multipart::MultipartError),

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
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        use AppError::{
            Anyhow, AskamaError, HeaderError, InternalError, MultipartError, NotFound,
            RSpotifyClientError, RSpotifyIdError, Sqlx, StrumError, UrlParseError, Utf8Error,
        };

        match self {
            NotFound => (StatusCode::NOT_FOUND).into_response(),
            InternalError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            Anyhow(err) => {
                warn!("anyhow: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            Sqlx(err) => {
                warn!("sqlx: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            MultipartError(err) => {
                warn!("multipart: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            Utf8Error(err) => {
                warn!("utf8: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            StrumError(err) => {
                warn!("strum: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            RSpotifyIdError(err) => {
                warn!("rspotify: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            RSpotifyClientError(err) => {
                warn!("rspotify: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            UrlParseError(err) => {
                warn!("url parser: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            HeaderError(err) => {
                warn!("header error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            AskamaError(err) => {
                warn!("askama error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
        }
    }
}
