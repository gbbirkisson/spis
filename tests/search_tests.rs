use spis::db;
mod common;

#[tokio::test]
async fn test_collections_search() {
    common::init_tracing();
    let (pool, _file) = common::setup_db().await;

    let base_dir = "/home/gbb/repos/personal/spis/data/media";

    // Insert some media in different folders
    let paths = vec![
        format!("{}/vacation/beach/img1.jpg", base_dir),
        format!("{}/vacation/mountain/img2.jpg", base_dir),
        format!("{}/work/office/img3.jpg", base_dir),
        format!("{}/work/remote/img4.jpg", base_dir),
        format!("{}/mountain_trip.jpg", base_dir),
    ];

    for path in paths {
        let id = uuid::Uuid::new_v4();
        let media = spis::media::ProcessedMedia {
            uuid: id,
            path,
            data: Some(spis::media::ProcessedMediaData {
                taken_at: chrono::Utc::now(),
            }),
            media_type: spis::media::ProcessedMediaType::Image,
        };
        db::media_insert(&pool, media).await.expect("Insert failed");
    }

    // Search for "mountain"
    let results = db::collections_search(&pool, base_dir, "mountain")
        .await
        .expect("Search failed");

    // Should match "vacation/mountain/"
    // "mountain_trip.jpg" is a file, so it should be ignored if we are looking for directories.
    // Wait, my SQL search for directories by looking for slashes AFTER the term.

    assert!(results.contains(&"vacation/mountain/".to_string()));
    assert!(!results.contains(&"mountain_trip.jpg".to_string()));
    assert_eq!(results.len(), 1);

    // Search for "vacation"
    let results = db::collections_search(&pool, base_dir, "vacation")
        .await
        .expect("Search failed");

    // Should match "vacation/", "vacation/beach/", "vacation/mountain/"
    assert!(results.contains(&"vacation/".to_string()));
    assert!(results.contains(&"vacation/beach/".to_string()));
    assert!(results.contains(&"vacation/mountain/".to_string()));
    assert_eq!(results.len(), 3);

    // Fuzzy search: "mt" should match "mountain"
    let results = db::collections_search(&pool, base_dir, "mt")
        .await
        .expect("Search failed");
    assert!(results.contains(&"vacation/mountain/".to_string()));

    // Fuzzy search: "v b" should match "vacation/beach/"
    let results = db::collections_search(&pool, base_dir, "v b")
        .await
        .expect("Search failed");
    assert!(results.contains(&"vacation/beach/".to_string()));

    // Search for "/" should include "/" at the top
    let results = db::collections_search(&pool, base_dir, "/")
        .await
        .expect("Search failed");
    assert_eq!(results[0], "/");
}
