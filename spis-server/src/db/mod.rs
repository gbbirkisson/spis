use chrono::{DateTime, Utc};
use eyre::{eyre, Result};
use std::path::PathBuf;

use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};

use crate::img::ProcessedImage;

pub async fn setup_db(db_file: PathBuf) -> Result<Pool<Sqlite>> {
    // Ensure db exits
    let db_file = db_file.to_str().ok_or(eyre!("Unable to get db file"))?;
    if !Sqlite::database_exists(db_file).await.unwrap_or(false) {
        Sqlite::create_database(db_file).await?;
    }

    // Create pool and run migrations
    let pool = SqlitePool::connect(db_file).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub async fn image_insert(pool: &SqlitePool, img: ProcessedImage) -> Result<()> {
    // Get image path
    let image = img
        .image
        .to_str()
        .ok_or(eyre!("Unable to get image path"))?;

    // Create query
    let query = match &img.data {
        Some(data) => {
            sqlx::query!(
                r#"
                INSERT OR REPLACE INTO images ( id, image, created_at, modified_at, walked )
                VALUES ( ?1, ?2, ?3, ?4, 1 )
                "#,
                img.uuid,
                image,
                data.created_at,
                data.modified_at,
            )
        }
        None => {
            sqlx::query!(
                r#"
                UPDATE images SET walked = 1, image = ?1 WHERE ID = ?2
                "#,
                image,
                img.uuid,
            )
        }
    };

    // Execute query
    query.execute(pool).await?;
    Ok(())
}

pub async fn image_mark_unwalked(pool: &SqlitePool) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE images SET walked = 0
        "#
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn image_delete_unwalked(pool: &SqlitePool) -> Result<u64> {
    let res = sqlx::query!(
        r#"
        DELETE FROM images where walked = 0
        "#
    )
    .execute(pool)
    .await?;
    Ok(res.rows_affected())
}

pub async fn image_count(pool: &SqlitePool) -> Result<i32> {
    let res = sqlx::query!(
        r#"
        SELECT count(*) as count FROM IMAGES
        "#
    )
    .fetch_one(pool)
    .await?;
    Ok(res.count)
}

#[derive(sqlx::FromRow)]
pub struct ImgRow {
    pub id: uuid::Uuid,
    pub image: String,
}

pub async fn image_get(
    pool: &SqlitePool,
    limit: i32,
    prev: Option<DateTime<Utc>>,
) -> Result<Vec<ImgRow>> {
    let query = match prev {
        None => sqlx::query_as::<Sqlite, ImgRow>(
            r#"
            SELECT id, image FROM images
            ORDER BY created_at DESC
            LIMIT ?
            "#,
        )
        .bind(limit),
        Some(_) => todo!(),
    };

    query
        .fetch_all(pool)
        .await
        .map_err(|e| eyre!("Failed to fetch rows: {e}"))
}
