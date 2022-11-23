use chrono::{DateTime, Utc};
use eyre::{eyre, Result};

use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};

use crate::media::ProcessedMedia;

pub async fn setup_db(db_file: &str) -> Result<Pool<Sqlite>> {
    tracing::info!("Setup db: {:?}", db_file);

    tracing::debug!("Ensure db exists");
    if !Sqlite::database_exists(db_file).await.unwrap_or(false) {
        Sqlite::create_database(db_file).await?;
    }

    tracing::debug!("Create pool and run migrations");
    let pool = SqlitePool::connect(db_file).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    tracing::debug!("DB setup: {:?}", db_file);
    Ok(pool)
}

pub async fn media_insert(pool: &SqlitePool, processed_media: ProcessedMedia) -> Result<()> {
    // Get media path
    let media_path = processed_media
        .media
        .to_str()
        .ok_or(eyre!("Unable to get media path"))?;

    // Create query
    let query = match &processed_media.data {
        Some(data) => {
            sqlx::query!(
                r#"
                INSERT OR REPLACE INTO media ( id, media, taken_at, walked )
                VALUES ( ?1, ?2, ?3, 1 )
                "#,
                processed_media.uuid,
                media_path,
                data.taken_at,
            )
        }
        None => {
            sqlx::query!(
                r#"
                UPDATE media SET walked = 1, media = ?1 WHERE ID = ?2
                "#,
                media_path,
                processed_media.uuid,
            )
        }
    };

    // Execute query
    query.execute(pool).await?;
    Ok(())
}

pub async fn media_mark_unwalked(pool: &SqlitePool) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE media SET walked = 0
        "#
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn media_delete_unwalked(pool: &SqlitePool) -> Result<u64> {
    let res = sqlx::query!(
        r#"
        DELETE FROM media where walked = 0
        "#
    )
    .execute(pool)
    .await?;
    Ok(res.rows_affected())
}

pub async fn media_count(pool: &SqlitePool) -> Result<i32> {
    let res = sqlx::query!(
        r#"
        SELECT count(*) as count FROM media
        "#
    )
    .fetch_one(pool)
    .await?;
    Ok(res.count)
}

#[derive(sqlx::FromRow)]
pub struct MediaRow {
    pub id: uuid::Uuid,
    pub media: String,
    pub taken_at: DateTime<Utc>,
}

pub async fn media_get(
    pool: &SqlitePool,
    limit: i32,
    taken_after: Option<DateTime<Utc>>,
) -> Result<Vec<MediaRow>> {
    let query = match taken_after {
        None => sqlx::query_as::<Sqlite, MediaRow>(
            r#"
            SELECT id, media, taken_at FROM media
            ORDER BY taken_at DESC
            LIMIT ?
            "#,
        )
        .bind(limit),
        Some(taken_after) => sqlx::query_as::<Sqlite, MediaRow>(
            r#"
            SELECT id, media, taken_at FROM media
            WHERE taken_at < ?
            ORDER BY taken_at DESC
            LIMIT ?
            "#,
        )
        .bind(taken_after)
        .bind(limit),
    };

    query
        .fetch_all(pool)
        .await
        .map_err(|e| eyre!("Failed to fetch rows: {e}"))
}
