use sqlx::PgPool;
use strum::{AsRefStr, Display, EnumString};

pub mod handlers;

#[derive(Debug, sqlx::Type, AsRefStr, EnumString, Display)]
#[sqlx(type_name = "category_type", rename_all = "lowercase")]
pub enum CategoryType {
    #[strum(serialize = "music")]
    Music,

    #[strum(serialize = "audiobook")]
    Audiobook,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Category {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub image_url: String,
    pub category_type: CategoryType,
}

async fn list_all_by_type(
    db: &PgPool,
    category_type: &CategoryType,
) -> anyhow::Result<Vec<Category>> {
    let result = sqlx::query_as!(
        Category,
        r#"
        SELECT 
            id, name, image_url, category_type AS "category_type!: CategoryType" 
        FROM categories
        WHERE category_type = ($1::text)::category_type
        "#,
        category_type.as_ref()
    )
    .fetch_all(db)
    .await?;

    Ok(result)
}

async fn list_all(db: &PgPool) -> anyhow::Result<Vec<Category>> {
    let result = sqlx::query_as!(
        Category,
        r#"
        SELECT 
            id, name, image_url, category_type AS "category_type!: CategoryType" 
        FROM categories
        "#,
    )
    .fetch_all(db)
    .await?;

    Ok(result)
}

async fn get(db: &PgPool, category_id: &str) -> anyhow::Result<Category> {
    let id = sqlx::types::Uuid::parse_str(category_id)?;
    let result = sqlx::query_as!(
        Category,
        r#"
        SELECT 
            id, name, image_url, category_type AS "category_type!: CategoryType" 
        FROM categories 
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(db)
    .await?;

    Ok(result)
}
