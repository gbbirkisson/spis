use std::{fmt::Error, path::PathBuf};

use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};

use crate::img::ProcessedImage;

pub async fn setup_db(db_file: PathBuf) -> Result<Pool<Sqlite>, Error> {
    let db_file = db_file.to_str().unwrap();
    if !Sqlite::database_exists(&db_file).await.unwrap_or(false) {
        Sqlite::create_database(&db_file).await.unwrap();
    }
    let pool = SqlitePool::connect(db_file).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    Ok(pool)
}

pub async fn insert_image(pool: &SqlitePool, img: ProcessedImage) {
    let id = sqlx::query!(
        r#"
        INSERT INTO images ( id )
        VALUES ( ?1 )
        "#,
        img.hash,
    )
    .execute(pool)
    .await
    .unwrap();
}
