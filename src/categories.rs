use sqlx::{PgPool, types::Uuid};
use strum::{AsRefStr, Display, EnumString};

pub mod handlers;

#[derive(Debug, sqlx::Type, AsRefStr, EnumString, PartialEq, Display)]
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
    pub visible: bool,
}

async fn list_all_by_type(
    db: &PgPool,
    category_type: &CategoryType,
) -> anyhow::Result<Vec<Category>> {
    let result = sqlx::query_as!(
        Category,
        r#"
        SELECT 
            c.id, c.name, c.image_url, c.category_type AS "category_type!: CategoryType", c.visible
        FROM categories AS c
        WHERE c.category_type = ($1::text)::category_type
            AND c.visible = TRUE
            AND (SELECT COUNT(e.id)
                    FROM entries AS e
                    WHERE e.visible = TRUE AND e.category_id = c.id
                ) > 0
        ORDER BY c.name
        "#,
        category_type.as_ref()
    )
    .fetch_all(db)
    .await?;

    Ok(result)
}

pub async fn list_all(db: &PgPool) -> anyhow::Result<Vec<Category>> {
    let result = sqlx::query_as!(
        Category,
        r#"
        SELECT 
            id, name, image_url, category_type AS "category_type!: CategoryType", visible
        FROM categories
        ORDER BY name
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
            id, name, image_url, category_type AS "category_type!: CategoryType", visible
        FROM categories 
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(db)
    .await?;

    Ok(result)
}

async fn delete(db: &PgPool, category_id: &str) -> anyhow::Result<()> {
    let id = sqlx::types::Uuid::parse_str(category_id)?;
    sqlx::query!(
        r#"
        DELETE FROM categories 
        WHERE id = $1
        "#,
        id
    )
    .execute(db)
    .await?;

    Ok(())
}

async fn update(db: &PgPool, category: &Category) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE categories
        SET
            name = $2, image_url = $3, category_type = ($4::text)::category_type, visible = $5
        WHERE id = $1
        "#,
        category.id,
        category.name,
        category.image_url,
        category.category_type.as_ref(),
        category.visible
    )
    .execute(db)
    .await?;

    Ok(())
}

async fn create(
    db: &PgPool,
    name: &str,
    image_url: &str,
    category_type: &CategoryType,
) -> anyhow::Result<Uuid> {
    let rec = sqlx::query!(
        r#"
        INSERT INTO categories (name, image_url, category_type)
        VALUES ($1, $2, ($3::text)::category_type)
        RETURNING id
        "#,
        name,
        image_url,
        category_type.as_ref()
    )
    .fetch_one(db)
    .await?;
    Ok(rec.id)
}

pub async fn admin_search(db: &PgPool, query: &str) -> anyhow::Result<Vec<Category>> {
    let query = format!("%{}%", query);
    let result = sqlx::query_as!(
        Category,
        r#"
        SELECT 
            id, name, image_url, category_type AS "category_type!: CategoryType", visible
        FROM categories
        WHERE name like $1
        ORDER BY name
        "#,
        query
    )
    .fetch_all(db)
    .await?;

    Ok(result)
}

pub async fn search(
    db: &PgPool,
    category_type: &CategoryType,
    query: &str,
) -> anyhow::Result<Vec<Category>> {
    let query = format!("%{}%", query);
    let result = sqlx::query_as!(
        Category,
        r#"
        SELECT 
            c.id, c.name, c.image_url, c.category_type AS "category_type!: CategoryType", c.visible
        FROM categories AS c
        WHERE c.category_type = ($1::text)::category_type
            AND c.visible = TRUE
            AND (SELECT COUNT(e.id)
                    FROM entries AS e
                    WHERE e.visible = TRUE AND e.category_id = c.id
                ) > 0
            AND c.name LIKE $2
        ORDER BY c.name
        "#,
        category_type.as_ref(),
        query
    )
    .fetch_all(db)
    .await?;

    Ok(result)
}
