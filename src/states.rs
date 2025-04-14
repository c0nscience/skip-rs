use rspotify::ClientCredsSpotify;
use sqlx::PgPool;

use crate::ha::Client;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: PgPool,
    pub spotify: ClientCredsSpotify,
    pub ha_client: Client,
}
