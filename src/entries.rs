use std::collections::HashMap;

use sqlx::{PgPool, types::Uuid};
use strum::{AsRefStr, Display, EnumString};

pub mod handlers;

#[derive(Debug, sqlx::Type, AsRefStr, EnumString, PartialEq, Display, Clone)]
#[sqlx(type_name = "entry_type", rename_all = "lowercase")]
pub enum EntryType {
    #[strum(serialize = "album")]
    Album,

    #[strum(serialize = "playlist")]
    Playlist,
}

async fn list_all_visible_by_category(
    db: &PgPool,
    category_id: &str,
) -> anyhow::Result<Vec<EntryListModel>> {
    let id = sqlx::types::Uuid::parse_str(category_id)?;
    let result = sqlx::query_as!(
        EntryListModel,
        r#"
        SELECT 
            id, name, image_url, visible, play_count as "play_count!"
        FROM entries
        WHERE category_id = $1 AND visible = TRUE
        ORDER BY name
        "#,
        id
    )
    .fetch_all(db)
    .await?;

    Ok(result)
}

pub async fn list_all_by_category(
    db: &PgPool,
    category_id: &str,
) -> anyhow::Result<Vec<EntryListModel>> {
    let id = sqlx::types::Uuid::parse_str(category_id)?;
    let result = sqlx::query_as!(
        EntryListModel,
        r#"
        SELECT 
            id, name, image_url, visible, play_count as "play_count!"
        FROM entries
        WHERE category_id = $1
        ORDER BY name
        "#,
        id
    )
    .fetch_all(db)
    .await?;

    Ok(result)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CategoryListModel {
    pub name: String,
    pub entries: Vec<EntryListModel>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntryListModel {
    pub id: String,
    pub name: String,
    pub image_url: String,
    pub visible: bool,
    pub play_count: i16,
}

async fn list_all(db: &PgPool) -> anyhow::Result<Vec<CategoryListModel>> {
    let result = sqlx::query!(
        r#"
        SELECT e.id AS "entry_id", e.name AS "entry_name", e.image_url AS "entry_image_url", e.visible AS "entry_visible", e.play_count AS "entry_play_count!", c.id AS "category_id?", c.name AS "catgegory_name?"
        FROM entries AS e
        LEFT OUTER JOIN categories AS c ON e.category_id = c.id
        ORDER BY e.name
        "#)
        .fetch_all(db)
        .await?;

    let mut categories: Vec<CategoryListModel> = result
        .iter()
        .fold(
            HashMap::new(),
            |mut acc, r| -> HashMap<Option<String>, CategoryListModel> {
                match acc.get_mut(&r.category_id.map(|id| id.to_string())) {
                    Some(category) => category.entries.push(EntryListModel {
                        id: r.entry_id.to_string(),
                        name: r.entry_name.clone(),
                        image_url: r.entry_image_url.clone(),
                        visible: r.entry_visible,
                        play_count: r.entry_play_count,
                    }),
                    None => {
                        acc.insert(
                            r.category_id.map(|id| id.to_string()),
                            CategoryListModel {
                                name: r.catgegory_name.clone().unwrap_or_else(|| "".to_string()),
                                entries: vec![EntryListModel {
                                    id: r.entry_id.to_string(),
                                    name: r.entry_name.clone(),
                                    image_url: r.entry_image_url.clone(),
                                    visible: r.entry_visible,
                                    play_count: r.entry_play_count,
                                }],
                            },
                        );
                        ()
                    }
                };

                acc
            },
        )
        .values()
        .cloned()
        .collect();

    categories.sort();

    Ok(categories)
}

#[derive(Debug, sqlx::FromRow)]
pub struct EntryEditModel {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub image_url: String,
    pub entry_type: EntryType,
    pub spotify_uri: String,
    pub spotify_id: String,
    pub play_count: i16,
    pub blob: serde_json::Value,
    pub category_id: Option<sqlx::types::Uuid>,
    pub visible: bool,
    pub spotify_url: String,
}

pub async fn get(db: &PgPool, entry_id: &str) -> anyhow::Result<EntryEditModel> {
    let id = sqlx::types::Uuid::parse_str(entry_id)?;
    let result = sqlx::query_as!(
        EntryEditModel,
        r#"
        SELECT id, name, image_url, entry_type AS "entry_type!: EntryType", spotify_uri, spotify_id, play_count AS "play_count!", blob, category_id, visible, spotify_url
        FROM entries
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(db)
    .await?;

    Ok(result)
}

async fn update(db: &PgPool, entry: &EntryEditModel) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE entries
        SET
            name = $2,
            image_url = $3,
            entry_type = ($4::text)::entry_type,
            spotify_uri = $5,
            spotify_id = $6,
            play_count = $7,
            blob = $8,
            category_id = $9,
            visible = $10
        WHERE id = $1
        "#,
        entry.id,
        entry.name,
        entry.image_url,
        entry.entry_type.as_ref(),
        entry.spotify_uri,
        entry.spotify_id,
        entry.play_count,
        entry.blob,
        entry.category_id,
        entry.visible
    )
    .execute(db)
    .await?;

    Ok(())
}

async fn delete(db: &PgPool, entry_id: &str) -> anyhow::Result<()> {
    let id = sqlx::types::Uuid::parse_str(entry_id)?;
    sqlx::query!(
        r#"
        DELETE FROM entries 
        WHERE id = $1
        "#,
        id
    )
    .execute(db)
    .await?;

    Ok(())
}

#[derive(Debug, PartialEq, Clone)]
pub struct EntryCreateModel {
    pub name: String,
    pub image_url: String,
    pub entry_type: EntryType,
    pub spotify_url: String,
    pub spotify_uri: String,
    pub spotify_id: String,
    pub play_count: i16,
    pub blob: serde_json::Value,
    pub visible: bool,
    pub category_id: Option<sqlx::types::Uuid>,
}

pub async fn create(db: &PgPool, entry: EntryCreateModel) -> anyhow::Result<Uuid> {
    let rec = sqlx::query!(
        r#"
        INSERT INTO entries (name, image_url, entry_type, spotify_uri, spotify_id, play_count, blob, visible, category_id, spotify_url)
        VALUES ($1, $2, ($3::text)::entry_type, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id
        "#,
        entry.name,
        entry.image_url,
        entry.entry_type.as_ref(),
        entry.spotify_uri,
        entry.spotify_id,
        entry.play_count,
        entry.blob,
        entry.visible,
        entry.category_id,
        entry.spotify_url
    )
    .fetch_one(db)
    .await?;
    Ok(rec.id)
}

pub async fn increment_play_count(db: &PgPool, entry_id: &str) -> anyhow::Result<()> {
    let id = sqlx::types::Uuid::parse_str(entry_id)?;
    sqlx::query!(
        r#"
        UPDATE entries
        SET
            play_count = play_count + 1
        WHERE id = $1
        "#,
        id
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn admin_search(db: &PgPool, query: &str) -> anyhow::Result<Vec<CategoryListModel>> {
    let query = format!("%{}%", query);
    let result = sqlx::query!(
        r#"
        SELECT e.id AS "entry_id", e.name AS "entry_name", e.image_url AS "entry_image_url", e.visible AS "entry_visible", e.play_count AS "entry_play_count!", c.id AS "category_id?", c.name AS "catgegory_name?"
        FROM entries AS e
        LEFT OUTER JOIN categories AS c ON e.category_id = c.id
        WHERE e.name LIKE $1
        ORDER BY e.name
        "#,
        query)
        .fetch_all(db)
        .await?;

    let mut categories: Vec<CategoryListModel> = result
        .iter()
        .fold(
            HashMap::new(),
            |mut acc, r| -> HashMap<Option<String>, CategoryListModel> {
                match acc.get_mut(&r.category_id.map(|id| id.to_string())) {
                    Some(category) => category.entries.push(EntryListModel {
                        id: r.entry_id.to_string(),
                        name: r.entry_name.clone(),
                        image_url: r.entry_image_url.clone(),
                        visible: r.entry_visible,
                        play_count: r.entry_play_count,
                    }),
                    None => {
                        acc.insert(
                            r.category_id.map(|id| id.to_string()),
                            CategoryListModel {
                                name: r.catgegory_name.clone().unwrap_or_else(|| "".to_string()),
                                entries: vec![EntryListModel {
                                    id: r.entry_id.to_string(),
                                    name: r.entry_name.clone(),
                                    image_url: r.entry_image_url.clone(),
                                    visible: r.entry_visible,
                                    play_count: r.entry_play_count,
                                }],
                            },
                        );
                        ()
                    }
                };

                acc
            },
        )
        .values()
        .cloned()
        .collect();

    categories.sort();

    Ok(categories)
}

pub async fn search(
    db: &PgPool,
    category_id: &str,
    query: &str,
) -> anyhow::Result<Vec<EntryListModel>> {
    let query = format!("%{}%", query);
    let id = sqlx::types::Uuid::parse_str(category_id)?;
    let result = sqlx::query_as!(
        EntryListModel,
        r#"
        SELECT 
            id, name, image_url, visible, play_count as "play_count!"
        FROM entries
        WHERE name LIKE $1 AND visible = TRUE AND category_id = $2
        ORDER BY name
        "#,
        query,
        id
    )
    .fetch_all(db)
    .await?;

    Ok(result)
}
