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
    pub category_type: CategoryType,
}

async fn list_all(db: &PgPool, category_type: &CategoryType) -> anyhow::Result<Vec<Category>> {
    let result = sqlx::query_as!(
        Category,
        r#"
        SELECT 
            id, name, category_type AS "category_type!: CategoryType" 
        FROM categories
        WHERE category_type = ($1::text)::category_type
        "#,
        category_type.as_ref()
    )
    .fetch_all(db)
    .await?;

    Ok(result)
}
