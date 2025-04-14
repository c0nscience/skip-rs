use std::collections::HashMap;

use axum::http::HeaderMap;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use tracing::error;

use crate::entries::{self, handlers::Room};

#[derive(Debug, Clone)]
pub struct Client {
    host: String,
    token: String,
}

impl Client {
    pub fn new(host: &str, token: &str) -> Client {
        Client {
            host: host.to_owned(),
            token: token.to_owned(),
        }
    }

    fn entity_id(&self, room: &entries::handlers::Room) -> String {
        let room = match room {
            entries::handlers::Room::Playroom => "playroom",
            entries::handlers::Room::Bathroom => "bathroom",
            entries::handlers::Room::Kitchen => "kitchen",
            entries::handlers::Room::LivingRoom => "living_room",
        };
        format!("media_player.{}", room)
    }

    fn url(&self, service: &str) -> String {
        format!("{}/api/services/media_player/{}", self.host, service)
    }

    fn headers(&self) -> anyhow::Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        headers.insert(AUTHORIZATION, format!("Bearer {}", self.token).parse()?);
        Ok(headers)
    }

    pub async fn play(
        &self,
        room: &entries::handlers::Room,
        spotify_id: &str,
    ) -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("entity_id".to_string(), self.entity_id(room));
        body.insert("media_content_id".to_string(), spotify_id.to_string());
        body.insert("media_content_type".to_string(), "playlist".to_string());
        body.insert("enqueue".to_string(), "replace".to_string());

        let res = client
            .post(self.url("play_media"))
            .headers(self.headers()?)
            .json(&body)
            .send()
            .await?;

        if let Err(err) = res.error_for_status() {
            error!("could not send request to home assist: {}", err)
        }
        Ok(())
    }

    pub async fn available_rooms(&self) -> anyhow::Result<Vec<Room>> {
        let client = reqwest::Client::new();
        let res = client
            .get(format!("{}/api/states", self.host))
            .headers(self.headers()?)
            .send()
            .await?;

        let body = res.json::<serde_json::Value>().await?;

        return if let Some(room_ids) = body.as_array().map(|a| {
            a.into_iter()
                .filter(|&e| {
                    e["entity_id"]
                        .as_str()
                        .is_some_and(|e| e.starts_with("media_player"))
                        && e["state"].as_str().is_some_and(|e| e != "unavailable")
                })
                .map(|e| {
                    e["entity_id"]
                        .as_str()
                        .map_or("", |e| e.trim_start_matches("media_player."))
                })
                .map(|e| match e {
                    "playroom" => entries::handlers::Room::Playroom,
                    "kitchen" => entries::handlers::Room::Kitchen,
                    "living_room" => entries::handlers::Room::LivingRoom,
                    "bathroom" => entries::handlers::Room::Bathroom,
                    _ => {
                        error!("could not match {} to room enum", e);
                        entries::handlers::Room::Playroom
                    }
                })
                .collect::<Vec<_>>()
        }) {
            Ok(room_ids)
        } else {
            Ok(vec![])
        };
    }
}
