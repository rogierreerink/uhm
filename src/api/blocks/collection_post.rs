use crate::db::blocks2::{BlockNew, DbBlocks};
use crate::db::Db;
use crate::global::AppState;
use crate::types::payloads::collection;
use axum::http::StatusCode;
use axum::{extract::State, response::IntoResponse, Json};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
struct Response<T> {
    data: Vec<T>,
}

#[axum::debug_handler]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(block): Json<collection::PostRequest<BlockNew>>,
) -> impl IntoResponse {
    let mut db_blocks = match state.db().blocks().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let blocks = match db_blocks.create(&block.data).await {
        Ok(blocks) => blocks,
        Err(err) => {
            tracing::error!("failed to create blocks: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((StatusCode::CREATED, Json(Response { data: blocks })))
}
