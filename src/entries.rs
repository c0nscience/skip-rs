use sqlx::PgPool;

pub mod handlers;

#[derive(Debug, sqlx::FromRow)]
pub struct Entry {
    id: sqlx::types::Uuid,
    name: String,
}

async fn list_all(db: &PgPool, category_id: &str) -> anyhow::Result<Vec<Entry>> {
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
