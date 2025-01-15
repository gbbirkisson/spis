use std::{collections::HashMap, fmt::Display};

use crate::media::{ProcessedMedia, ProcessedMediaType};
use chrono::{DateTime, Utc};
use color_eyre::{eyre::Context, Result};
use sqlx::{
    migrate::MigrateDatabase, query::QueryAs, sqlite::SqliteArguments, Pool, Sqlite, SqlitePool,
};

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

#[allow(clippy::module_name_repetitions)]
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

pub async fn media_insert(pool: &SqlitePool, processed_media: ProcessedMedia) -> Result<()> {
    let media_type = processed_media.media_type.convert();
    match &processed_media.data {
        Some(data) => match data.pos {
            Some(p) => {
                sqlx::query!(
                    r#"
                    INSERT OR REPLACE INTO media ( id, path, taken_at, type, latitude, longitude, walked )
                    VALUES ( ?1, ?2, ?3, ?4,  ?5,  ?6, 1 )
                    "#,
                    processed_media.uuid,
                    processed_media.path,
                    data.taken_at,
                    media_type,
                    p.0,
                    p.1,
                ).execute(pool).await?
            }
            None => {
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
                .execute(pool)
                .await?
            }
        },
        None => {
            sqlx::query!(
                r#"
                UPDATE media SET walked = 1, path = ?2 WHERE ID = ?1
                "#,
                processed_media.uuid,
                processed_media.path,
            )
            .execute(pool)
            .await?
        }
    };
    Ok(())
}

#[derive(sqlx::FromRow)]
pub struct MediaHashRow {
    pub id: uuid::Uuid,
    pub path: String,
}

pub async fn media_hashmap(pool: &SqlitePool) -> Result<HashMap<String, uuid::Uuid>> {
    tracing::debug!("Collect all DB entries UUIDs");
    let res = sqlx::query_as::<Sqlite, MediaHashRow>(
        r"
        SELECT id, path FROM media
        ",
    )
    .fetch_all(pool)
    .await?;
    Ok(res.into_iter().map(|e| (e.path, e.id)).collect())
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
    pub pos: Option<i32>,
}

pub async fn media_count(pool: &SqlitePool) -> Result<MediaCount> {
    let res = sqlx::query_as::<Sqlite, MediaCount>(
        r"
        SELECT
        COUNT(*) as count,
        SUM(walked) as walked,
        SUM(favorite) as favorite,
        SUM(archived) as archived,
        SUM(missing) as missing,
        COUNT(latitude) as pos
        FROM media
        ",
    )
    .fetch_one(pool)
    .await?;
    Ok(res)
}

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

#[derive(sqlx::FromRow, Debug)]
pub struct MediaRow {
    pub id: uuid::Uuid,
    pub path: String,
    pub taken_at: DateTime<Utc>,
    pub media_type: i32,
    pub archived: bool,
    pub favorite: bool,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

pub enum Order {
    Asc,
    Desc,
}

impl Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Asc => f.write_str("ORDER BY taken_at ASC")?,
            Self::Desc => f.write_str("ORDER BY taken_at DESC")?,
        }
        Ok(())
    }
}

pub struct Filter {
    pub archived: bool,
    pub favorite: Option<bool>,
    pub taken_after: Option<DateTime<Utc>>,
    pub taken_before: Option<DateTime<Utc>>,
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WHERE NOT missing AND archived = ?")?;

        if self.favorite.is_some() {
            f.write_str(" AND favorite = ?")?;
        }

        if self.taken_after.is_some() {
            f.write_str(" AND taken_at > ?")?;
        }

        if self.taken_before.is_some() {
            f.write_str(" AND taken_at < ?")?;
        }

        Ok(())
    }
}

impl Filter {
    fn bind<'a>(
        &self,
        query: QueryAs<'a, Sqlite, MediaRow, SqliteArguments<'a>>,
    ) -> QueryAs<'a, Sqlite, MediaRow, SqliteArguments<'a>> {
        let mut query = query.bind(self.archived);

        if let Some(favorite) = self.favorite {
            query = query.bind(favorite);
        }

        if let Some(taken_after) = self.taken_after {
            query = query.bind(taken_after);
        }

        if let Some(taken_before) = self.taken_before {
            query = query.bind(taken_before);
        }

        query
    }
}

#[allow(clippy::future_not_send)]
pub async fn media_list(
    pool: &SqlitePool,
    filter: impl Into<Filter>,
    order: impl Into<Order>,
    limit: usize,
) -> Result<Vec<MediaRow>> {
    let filter = filter.into();
    let order = order.into();

    let query = format!(
        r"
SELECT id, path, taken_at, type as media_type, archived, favorite, latitude, longitude FROM media
{filter}
{order}
LIMIT ?
"
    );
    let mut query = sqlx::query_as::<Sqlite, MediaRow>(&query);
    query = filter.bind(query);
    query = query.bind(i32::try_from(limit).expect("Failed to convert limit"));
    query.fetch_all(pool).await.wrap_err("Failed to fetch rows")
}

#[allow(clippy::future_not_send)]
pub async fn media_with_pos(pool: &SqlitePool) -> Result<Vec<MediaRow>> {
    let query = r"
        SELECT id, path, taken_at, type as media_type, archived, favorite, latitude, longitude FROM media
        WHERE latitude IS NOT NULL;
        "
    .to_string();
    let query = sqlx::query_as::<Sqlite, MediaRow>(&query);
    query.fetch_all(pool).await.wrap_err("Failed to fetch rows")
}

#[allow(clippy::future_not_send)]
pub async fn media_get(
    pool: &SqlitePool,
    filter: impl Into<Filter>,
    order: impl Into<Order>,
    uuid: &uuid::Uuid,
) -> Result<(Option<MediaRow>, Option<MediaRow>, Option<MediaRow>)> {
    let filter = filter.into();
    let order = order.into();

    let query = format!(
        r"
WITH NR_MED AS (
    SELECT *, ROW_NUMBER() OVER ({order}) AS RN FROM media
    {filter}
)
SELECT id, path, taken_at, type as media_type, archived, favorite, latitude, longitude FROM NR_MED
WHERE RN IN (
	SELECT RN+i
    FROM NR_MED
    CROSS JOIN (SELECT -1 AS i UNION ALL SELECT 0 UNION ALL SELECT 1) n
    WHERE id = ?
)
"
    );
    let mut query = sqlx::query_as::<Sqlite, MediaRow>(&query);
    query = filter.bind(query);
    query = query.bind(uuid);
    let res = query
        .fetch_all(pool)
        .await
        .wrap_err("Failed to fetch rows")?;
    match res.len() {
        1 => {
            let mut res = res.into_iter();
            Ok((None, res.next(), None))
        }
        2 => {
            let mut res = res.into_iter();
            let a = res.next().expect("First media should be there");
            let b = res.next().expect("Second media should be there");
            if a.id == *uuid {
                Ok((None, Some(a), Some(b)))
            } else {
                Ok((Some(a), Some(b), None))
            }
        }
        3 => {
            let mut res = res.into_iter();
            Ok((res.next(), res.next(), res.next()))
        }
        _ => Err(color_eyre::eyre::eyre!("Media query returned bad result")),
    }
}
