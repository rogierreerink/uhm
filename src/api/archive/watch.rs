use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tracing::instrument;

use crate::global::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new().route("/", get(handle_ws)).with_state(state)
}

#[instrument(skip(state, ws))]
async fn handle_ws(State(state): State<Arc<AppState>>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_ws_socket(state, socket))
}

#[instrument(skip(state, socket))]
async fn handle_ws_socket(state: Arc<AppState>, socket: WebSocket) {
    tracing::info!("Handling new WebSocket connection");

    let (sender, receiver) = socket.split();
    tokio::spawn(handle_ws_read(receiver));
    tokio::spawn(handle_ws_write(state, sender));

    tracing::info!("Releasing WebSocket connection");
}

#[instrument(skip(socket))]
async fn handle_ws_read(mut socket: SplitStream<WebSocket>) {
    tracing::info!("Handling WebSocket read task");

    while let Some(message) = socket.next().await {
        match message {
            Ok(message) => tracing::info!("Got new data: {:?}", message),
            Err(err) => tracing::info!("Failed to read WebSocket message: {}", err),
        }
    }

    tracing::info!("Releasing WebSocket read task");
}

#[instrument(skip(state, socket))]
async fn handle_ws_write(state: Arc<AppState>, mut socket: SplitSink<WebSocket, Message>) {
    tracing::info!("Handling WebSocket write task");

    let mut change_receiver = state.change_notifier.subscribe();
    loop {
        match change_receiver.recv().await {
            Ok(path) => {
                if let Err(err) = socket.send(path.into()).await {
                    tracing::warn!("Failed to send notification to WebSocket client: {}", err);
                    break;
                }
            }
            Err(err) => tracing::warn!("Failed to receive notification: {}", err),
        }
    }

    tracing::info!("Releasing WebSocket write task");
}
