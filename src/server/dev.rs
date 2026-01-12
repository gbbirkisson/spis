use crate::server::AppState;
use axum::{
    Router,
    extract::{WebSocketUpgrade, ws::WebSocket},
    response::IntoResponse,
    routing::get,
};

async fn handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        if msg.is_err() {
            return;
        }
    }
}

pub fn create_router() -> Router<AppState> {
    Router::new().route("/ws", get(handler))
}
