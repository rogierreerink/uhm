use crate::db::blocks2::{BlockUpdate, DbBlocks};
use crate::db::Db;
use crate::global::AppState;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{extract::State, response::IntoResponse, Json};
use std::sync::Arc;
use uuid::Uuid;

#[axum::debug_handler]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(block): Json<BlockUpdate>,
) -> impl IntoResponse {
    let mut db_blocks = match state.db().blocks().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let block = match db_blocks.update(&id, &block).await {
        Ok(blocks) => blocks,
        Err(err) => {
            tracing::error!("failed to update block: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((StatusCode::OK, Json(block)))
}
