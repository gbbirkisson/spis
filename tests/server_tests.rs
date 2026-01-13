mod common;
use chrono::{Datelike, Utc};

#[tokio::test]
async fn test_server_index_redirect() {
    let (addr, _pool, _file) = common::spawn_server().await;
    let client = reqwest::Client::new();

    let response = client.get(&addr).send().await.expect("Request failed");
    assert!(response.status().is_success());
    assert_eq!(response.url().path(), "/hx");
}

#[tokio::test]
async fn test_gallery_page() {
    let (addr, pool, _file) = common::spawn_server().await;
    common::insert_dummy_media(&pool, 5).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/hx/gallery", addr))
        .send()
        .await
        .expect("Request failed");
    assert!(response.status().is_success());
    let text = response.text().await.expect("Failed to get text");
    assert!(text.contains("gallery"));
}

#[tokio::test]
async fn test_gallery_more() {
    let (addr, pool, _file) = common::spawn_server().await;
    common::insert_dummy_media(&pool, 5).await;
    let client = reqwest::Client::new();

    // Test /hx/gallery/more with cursor
    let now = Utc::now().to_rfc3339();
    let params = [("cursor", now)];
    let query = serde_urlencoded::to_string(params).expect("Failed to encode query");
    let response = client
        .get(format!("{}/hx/gallery/more?{}", addr, query))
        .send()
        .await
        .expect("Request failed");
    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_bar_endpoints() {
    let (addr, _pool, _file) = common::spawn_server().await;
    let client = reqwest::Client::new();

    // clear
    let response = client
        .get(format!("{}/hx/bar/clear", addr))
        .send()
        .await
        .expect("Request failed");
    assert!(response.status().is_success());

    // favorite
    let response = client
        .get(format!("{}/hx/bar/favorite", addr))
        .send()
        .await
        .expect("Request failed");
    assert!(response.status().is_success());

    // order
    let response = client
        .get(format!("{}/hx/bar/order", addr))
        .send()
        .await
        .expect("Request failed");
    assert!(response.status().is_success());

    // year
    let year = Utc::now().year();
    let response = client
        .get(format!("{}/hx/bar/year/{}", addr, year))
        .send()
        .await
        .expect("Request failed");
    assert!(response.status().is_success());

    // year toggle (same year)
    let response = client
        .get(format!("{}/hx/bar/year/{}?year={}", addr, year, year))
        .send()
        .await
        .expect("Request failed");
    assert!(response.status().is_success());

    // month
    let month = Utc::now().month();
    let response = client
        .get(format!("{}/hx/bar/month/{}?year={}", addr, month, year))
        .send()
        .await
        .expect("Request failed");
    assert!(response.status().is_success());

    // month toggle
    let response = client
        .get(format!(
            "{}/hx/bar/month/{}?year={}&month={}",
            addr, month, year, month
        ))
        .send()
        .await
        .expect("Request failed");
    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_preview_endpoints() {
    let (addr, pool, _file) = common::spawn_server().await;
    let uuids = common::insert_dummy_media(&pool, 3).await;
    let uuid = uuids[1]; // Use middle one to have prev and next
    let client = reqwest::Client::new();

    // root (view)
    let response = client
        .get(format!("{}/hx/preview/{}", addr, uuid))
        .send()
        .await
        .expect("Request failed");
    assert!(response.status().is_success());

    // favorite (put)
    let response = client
        .put(format!("{}/hx/preview/{}/favorite/true", addr, uuid))
        .send()
        .await
        .expect("Request failed");
    assert!(response.status().is_success());

    // archive (request confirmation)
    let response = client
        .get(format!("{}/hx/preview/{}?archive=true", addr, uuid))
        .send()
        .await
        .expect("Request failed");
    assert!(response.status().is_success());
    // Should show confirmation (red icon)
    let text = response.text().await.unwrap();
    assert!(text.contains("icon-red"));

    // confirm archive (delete)
    let response = client
        .delete(format!("{}/hx/preview/{}", addr, uuid))
        .send()
        .await
        .expect("Request failed");
    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_preview_error() {
    let (addr, _pool, _file) = common::spawn_server().await;
    let client = reqwest::Client::new();
    let uuid = uuid::Uuid::new_v4();

    // Request non-existent UUID should trigger DB error (as seen in analysis)
    let response = client
        .get(format!("{}/hx/preview/{}", addr, uuid))
        .send()
        .await
        .expect("Request failed");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
    let text = response.text().await.unwrap();
    assert!(text.contains("Media query returned bad result"));
}

#[tokio::test]
async fn test_assets() {
    let (addr, _pool, _file) = common::spawn_server().await;
    let client = reqwest::Client::new();

    // 1. favicon.png (image/png)
    let response = client
        .get(format!("{}/assets/favicon.png", addr))
        .send()
        .await
        .expect("Request failed");
    assert!(response.status().is_success());
    assert_eq!(response.headers().get("content-type").unwrap(), "image/png");

    // 2. manifest.json (application/json)
    let response = client
        .get(format!("{}/assets/manifest.json", addr))
        .send()
        .await
        .expect("Request failed");
    assert!(response.status().is_success());
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/json"
    );

    // 3. nested file: fa/css/brands.min.css (text/css)
    let response = client
        .get(format!("{}/assets/fa/css/all.min.css", addr))
        .send()
        .await
        .expect("Request failed");
    assert!(response.status().is_success());
    assert_eq!(response.headers().get("content-type").unwrap(), "text/css");

    // 4. nested file: scripts/dev-ws.js (application/javascript)
    let response = client
        .get(format!("{}/assets/scripts/dev-ws.js", addr))
        .send()
        .await
        .expect("Request failed");
    assert!(response.status().is_success());
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/javascript"
    );

    // 5. Non-existent file (404)
    let response = client
        .get(format!("{}/assets/does_not_exist.txt", addr))
        .send()
        .await
        .expect("Request failed");
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}
