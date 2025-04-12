use std::collections::HashMap;

use sqlx::{PgPool, types::Uuid};
use strum::{AsRefStr, Display, EnumString};

pub mod handlers;

#[derive(Debug, sqlx::Type, AsRefStr, EnumString, PartialEq, Display)]
#[sqlx(type_name = "entry_type", rename_all = "lowercase")]
pub enum EntryType {
    #[strum(serialize = "album")]
    Album,

    #[strum(serialize = "playlist")]
    Playlist,
}
#[derive(Debug, sqlx::FromRow)]
pub struct Entry {
    id: sqlx::types::Uuid,
    name: String,
}

async fn list_all_by_type(db: &PgPool, category_id: &str) -> anyhow::Result<Vec<Entry>> {
    let id = sqlx::types::Uuid::parse_str(category_id)?;
    let result = sqlx::query_as!(
        Entry,
        r#"
        SELECT 
            id, name 
        FROM entries
        WHERE category_id = $1
        "#,
        id
    )
    .fetch_all(db)
    .await?;

    Ok(result)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CategoryListModel {
    name: String,
    entries: Vec<EntryListModel>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct EntryListModel {
    id: String,
    name: String,
    image_url: String,
}

async fn list_all(db: &PgPool) -> anyhow::Result<Vec<CategoryListModel>> {
    let result = sqlx::query!(
        r#"
        SELECT e.id as "entry_id", e.name as "entry_name", e.image_url as "entry_image_url", c.id as "category_id", c.name as "catgegory_name"
        FROM entries as e
        LEFT OUTER JOIN categories as c ON e.category_id = c.id;
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
    id: sqlx::types::Uuid,
    name: String,
    image_url: String,
    entry_type: EntryType,
}

async fn get(db: &PgPool, entry_id: &str) -> anyhow::Result<EntryEditModel> {
    let id = sqlx::types::Uuid::parse_str(entry_id)?;
    let result = sqlx::query_as!(
        EntryEditModel,
        r#"
        SELECT id, name, image_url, entry_type AS "entry_type!: EntryType"
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
            entry_type = ($4::text)::entry_type
        WHERE id = $1
        "#,
        entry.id,
        entry.name,
        entry.image_url,
        entry.entry_type.as_ref(),
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

async fn create(
    db: &PgPool,
    name: &str,
    image_url: &str,
    entry_type: &EntryType,
) -> anyhow::Result<Uuid> {
    let rec = sqlx::query!(
        r#"
        INSERT INTO entries (name, image_url, entry_type)
        VALUES ($1, $2, ($3::text)::entry_type)
        RETURNING id
        "#,
        name,
        image_url,
        entry_type.as_ref()
    )
    .fetch_one(db)
    .await?;
    Ok(rec.id)
}
