use std::{fmt::Error, path::PathBuf};

use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};

use crate::img::ProcessedImage;

pub async fn setup_db(db_file: PathBuf) -> Result<Pool<Sqlite>, Error> {
    let db_file = db_file.to_str().unwrap();
    if !Sqlite::database_exists(db_file).await.unwrap_or(false) {
        Sqlite::create_database(db_file).await.unwrap();
    }
    let pool = SqlitePool::connect(db_file).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    Ok(pool)
}

pub async fn insert_image(pool: &SqlitePool, img: ProcessedImage) {
    let image = img.image.to_str().unwrap();
    let query = match &img.data {
        Some(data) => {
            sqlx::query!(
                r#"
                INSERT INTO images ( id, image, created_at, modified_at )
                VALUES ( ?1, ?2, ?3, ?4 )
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
                UPDATE images SET walked = 1, image = ?2 WHERE ID = ?1
                "#,
                img.uuid,
                image,
            )
        }
    };

    query.execute(pool).await.unwrap();
}
