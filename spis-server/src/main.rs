use std::{net::TcpListener, path::PathBuf};

use spis_server::{img, run, state::State};
use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt::init();

    let img_dir = PathBuf::from("dev/api/images");
    let thumb_dir = PathBuf::from("dev/api/state/thumbnails");

    let (tx, mut rx) = channel(32);
    img::image_processor(img_dir, thumb_dir, tx);

    tokio::spawn(async move {
        loop {
            rx.recv().await;
        }
    });

    let state = State::load("dev/api/state", "dev/api/images");
    let listener = TcpListener::bind("0.0.0.0:8000").expect("Failed to bind random port");
    let server = run(state, listener).expect("Failed to create server");
    server.await
}

// #[tokio::main]
// async fn main() {
//     let img_dir = PathBuf::from("dev/api/images");
//     let ext = vec!["jpg".to_string()];

//     let (s, r) = img::walker::start_walker(ext);

//     for id in 0..5 {
//         let r = r.clone();
//         tokio::spawn(async move {
//             loop {
//                 println!("Async task {} loop", id);
//                 match r.recv().await {
//                     Ok(msg) => {
//                         println!("Async task {} got {:?}", id, msg);
//                     }
//                     Err(e) => {
//                         println!("Async task error: {}", e);
//                     }
//                 }

//                 tokio::time::sleep(Duration::from_secs(1)).await;
//             }
//         });
//     }

//     s.send(img_dir).await.expect("should not fail");

//     tokio::time::sleep(Duration::from_secs(10)).await;
// }
