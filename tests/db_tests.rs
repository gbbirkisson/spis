use spis::{
    db,
    media::{ProcessedMedia, ProcessedMediaData, ProcessedMediaType},
};
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_media_lifecycle() {
    common::init_tracing();
    let (pool, _file) = common::setup_db().await;

    // 1. Insert Media
    let id = Uuid::new_v4();
    let media = ProcessedMedia {
        uuid: id,
        path: "/test/path.jpg".to_string(),
        data: Some(ProcessedMediaData {
            taken_at: chrono::Utc::now(),
        }),
        media_type: ProcessedMediaType::Image,
    };

    db::media_insert(&pool, media).await.expect("Insert failed");

    // 2. Get Media
    let (prev, current, next) = db::media_get(
        &pool,
        db::Filter {
            archived: false,
            favorite: None,
            taken_after: None,
            taken_before: None,
        },
        db::Order::Desc,
        &id,
    )
    .await
    .expect("Get failed");

    assert!(prev.is_none());
    assert!(next.is_none());
    assert!(current.is_some());
    let current = current.unwrap();
    assert_eq!(current.id, id);
    assert_eq!(current.path, "/test/path.jpg");
    assert!(!current.favorite);
    assert!(!current.archived);

    // 3. Mark Favorite
    let updated = db::media_favorite(&pool, &id, true)
        .await
        .expect("Favorite failed");
    assert!(updated);

    let (_, current, _) = db::media_get(
        &pool,
        db::Filter {
            archived: false,
            favorite: None,
            taken_after: None,
            taken_before: None,
        },
        db::Order::Desc,
        &id,
    )
    .await
    .expect("Get failed");
    assert!(current.unwrap().favorite);

    // 4. Archive
    let updated = db::media_archive(&pool, &id, true)
        .await
        .expect("Archive failed");
    assert!(updated);

    let (_, current, _) = db::media_get(
        &pool,
        db::Filter {
            archived: true, // It is now archived
            favorite: None,
            taken_after: None,
            taken_before: None,
        },
        db::Order::Desc,
        &id,
    )
    .await
    .expect("Get failed");
    assert!(current.unwrap().archived);
}

#[tokio::test]
async fn test_media_cleanup() {
    let (pool, _file) = common::setup_db().await;
    common::insert_dummy_media(&pool, 5).await;

    // Mark all as unwalked
    db::media_mark_unwalked(&pool)
        .await
        .expect("Mark unwalked failed");

    // Mark missing (since they are unwalked)
    let missing_count = db::media_mark_missing(&pool)
        .await
        .expect("Mark missing failed");
    assert_eq!(missing_count, 5);

    // Verify they are missing
    let count = db::media_count(&pool).await.expect("Count failed");
    assert_eq!(count.missing.unwrap(), 5);
}

#[tokio::test]
async fn test_media_list_pagination() {
    let (pool, _file) = common::setup_db().await;
    common::insert_dummy_media(&pool, 10).await;

    let list = db::media_list(
        &pool,
        db::Filter {
            archived: false,
            favorite: None,
            taken_after: None,
            taken_before: None,
        },
        db::Order::Desc,
        5,
    )
    .await
    .expect("List failed");

    assert_eq!(list.len(), 5);
}
