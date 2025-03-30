use crate::db::blocks2::DbBlocks;
use crate::db::Db;
use crate::global::AppState;
use axum::http::StatusCode;
use axum::{extract::State, response::IntoResponse, Json};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
struct Response<T> {
    data: T,
}

#[axum::debug_handler]
pub async fn handle(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut db_blocks = match state.db().blocks().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let blocks = match db_blocks.get_all().await {
        Ok(blocks) => blocks,
        Err(err) => {
            tracing::error!("failed to get blocks: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((StatusCode::OK, Json(Response { data: blocks })))
}
