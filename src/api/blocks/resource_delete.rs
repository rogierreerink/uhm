use crate::db::blocks2::DbBlocks;
use crate::db::{Db, DbError};
use crate::global::AppState;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;
use uuid::Uuid;

#[axum::debug_handler]
pub async fn handle(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let mut db_blocks = match state.db().blocks().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if let Err(err) = db_blocks.delete(&id).await {
        match err {
            DbError::NotFound => {
                tracing::error!("block could not be found");
                return Err(StatusCode::NOT_FOUND);
            }
            _ => {
                tracing::error!("failed to delete block: {}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    };

    Ok(StatusCode::OK)
}
