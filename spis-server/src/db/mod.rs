use std::collections::HashMap;

use crate::media::{ProcessedMedia, ProcessedMediaType};
use chrono::{DateTime, Utc};
use color_eyre::{eyre::Context, Result};
use spis_model::MediaType;
use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};

pub trait MediaTypeConverter<T> {
    fn convert(&self) -> T;
}

impl MediaTypeConverter<i32> for ProcessedMediaType {
    fn convert(&self) -> i32 {
        match self {
            Self::Image => 0,
            Self::Video => 1,
        }
    }
}

impl MediaTypeConverter<MediaType> for i32 {
    fn convert(&self) -> MediaType {
        match self {
            0 => MediaType::Image,
            1 => MediaType::Video,
            _ => unreachable!(),
        }
    }
}

#[allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]
pub async fn setup_db(db_file: &str) -> Result<Pool<Sqlite>> {
    tracing::debug!("Setup db");

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

#[allow(clippy::missing_errors_doc)]
pub async fn media_insert(pool: &SqlitePool, processed_media: ProcessedMedia) -> Result<()> {
    let media_type = processed_media.media_type.convert();
    match &processed_media.data {
        Some(data) => {
            sqlx::query!(
                r#"
                INSERT OR REPLACE INTO media ( id, path, taken_at, type, walked )
                VALUES ( ?1, ?2, ?3, ?4, 1 )
                "#,
                processed_media.uuid,
                processed_media.path,
                data.taken_at,
                media_type,
            )
        }
        None => {
            sqlx::query!(
                r#"
                UPDATE media SET walked = 1, path = ?2 WHERE ID = ?1
                "#,
                processed_media.uuid,
                processed_media.path,
            )
        }
    }
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(sqlx::FromRow)]
pub struct MediaHashRow {
    pub id: uuid::Uuid,
    pub path: String,
}

#[allow(clippy::missing_errors_doc)]
pub async fn media_hashmap(pool: &SqlitePool) -> Result<HashMap<String, uuid::Uuid>> {
    tracing::debug!("Collect all DB entries UUIDs");
    let res = sqlx::query_as::<Sqlite, MediaHashRow>(
        r#"
        SELECT id, path FROM media
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(res.into_iter().map(|e| (e.path, e.id)).collect())
}

#[allow(clippy::missing_errors_doc)]
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

#[allow(clippy::missing_errors_doc)]
pub async fn media_mark_missing(pool: &SqlitePool) -> Result<u64> {
    let res = sqlx::query!(
        r#"
        UPDATE media SET missing = 1 WHERE walked = 0;
        UPDATE media SET missing = 0 WHERE walked = 1;
        "#
    )
    .execute(pool)
    .await?;
    Ok(res.rows_affected())
}

#[derive(sqlx::FromRow, Debug)]
pub struct MediaCount {
    pub count: i32,
    pub walked: Option<i32>,
    pub favorite: Option<i32>,
    pub archived: Option<i32>,
    pub missing: Option<i32>,
}

#[allow(clippy::missing_errors_doc)]
pub async fn media_count(pool: &SqlitePool) -> Result<MediaCount> {
    let res = sqlx::query_as::<Sqlite, MediaCount>(
        r#"
        SELECT
        COUNT(*) as count,
        SUM(walked) as walked,
        SUM(favorite) as favorite,
        SUM(archived) as archived,
        SUM(missing) as missing
        FROM media
        "#,
    )
    .fetch_one(pool)
    .await?;
    Ok(res)
}

#[allow(clippy::missing_errors_doc)]
pub async fn media_archive(pool: &SqlitePool, uuid: &uuid::Uuid, archive: bool) -> Result<bool> {
    let res = sqlx::query!(
        r#"
        UPDATE media SET archived = ?2 WHERE id = ?1
        "#,
        uuid,
        archive,
    )
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

#[allow(clippy::missing_errors_doc)]
pub async fn media_favorite(pool: &SqlitePool, uuid: &uuid::Uuid, archive: bool) -> Result<bool> {
    let res = sqlx::query!(
        r#"
        UPDATE media SET favorite = ?2 WHERE id = ?1
        "#,
        uuid,
        archive,
    )
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

#[derive(sqlx::FromRow)]
pub struct MediaRow {
    pub id: uuid::Uuid,
    pub path: String,
    pub taken_at: DateTime<Utc>,
    pub media_type: i32,
    pub archived: bool,
    pub favorite: bool,
}

#[allow(clippy::missing_errors_doc)]
pub async fn media_get(
    pool: &SqlitePool,
    limit: i32,
    archived: bool,
    favorite: Option<bool>,
    taken_after: Option<DateTime<Utc>>,
    taken_before: Option<DateTime<Utc>>,
) -> Result<Vec<MediaRow>> {
    let mut query = String::new();

    query.push_str("SELECT id, path, taken_at, type as media_type, archived, favorite FROM media");
    query.push_str(" WHERE NOT missing AND archived = ?");

    if favorite.is_some() {
        query.push_str(" AND favorite = ?");
    }

    if taken_after.is_some() {
        query.push_str(" AND taken_at > ?");
    }

    if taken_before.is_some() {
        query.push_str(" AND taken_at < ?");
    }

    query.push_str(" ORDER BY taken_at DESC");
    query.push_str(" LIMIT ?");

    let mut query = sqlx::query_as::<Sqlite, MediaRow>(&query).bind(archived);

    if let Some(favorite) = favorite {
        query = query.bind(favorite);
    }

    if let Some(taken_after) = taken_after {
        query = query.bind(taken_after);
    }

    if let Some(taken_before) = taken_before {
        query = query.bind(taken_before);
    }

    query = query.bind(limit);

    query.fetch_all(pool).await.wrap_err("Failed to fetch rows")
}
