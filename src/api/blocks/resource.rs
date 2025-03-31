use crate::db::blocks::{BlockUpdate, DbBlocks};
use crate::db::Db;
use crate::global::AppState;
use crate::{api::handle_options, db::DbError};

use axum::{
    extract::{Path, State},
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{delete, get, options, patch},
    Json, Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::set_header::SetResponseHeaderLayer;
use tracing::instrument;
use uuid::Uuid;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/:id", get(get_resource))
        .route("/:id", patch(patch_resource))
        .route("/:id", delete(delete_resource))
        .route("/:id", options(handle_options))
        .layer(
            ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::if_not_present(
                    header::ACCESS_CONTROL_ALLOW_METHODS,
                    HeaderValue::from_static("GET, PATCH, DELETE, OPTIONS"),
                ))
                .layer(SetResponseHeaderLayer::if_not_present(
                    header::ACCESS_CONTROL_ALLOW_HEADERS,
                    HeaderValue::from_static("content-type"),
                )),
        )
        .with_state(state.clone())
}

#[axum::debug_handler]
#[instrument(skip(state))]
pub async fn get_resource(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let mut db_blocks = match state.db().blocks().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let block = match db_blocks.get_by_id(&id).await {
        Ok(block) => block,
        Err(err) => match err.downcast_ref::<DbError>() {
            Some(DbError::NotFound) => {
                tracing::error!("block could not be found: {:?}", err);
                return Err(StatusCode::NOT_FOUND);
            }
            _ => {
                tracing::error!("failed to get block: {:?}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
    };

    Ok((StatusCode::OK, Json(block)))
}

#[axum::debug_handler]
#[instrument(skip(state, block))]
pub async fn patch_resource(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(block): Json<BlockUpdate>,
) -> impl IntoResponse {
    let mut db_blocks = match state.db().blocks().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let block = match db_blocks.update(&id, &block).await {
        Ok(blocks) => blocks,
        Err(err) => match err.downcast_ref::<DbError>() {
            Some(DbError::NotFound) => {
                tracing::error!("block could not be found: {:?}", err);
                return Err(StatusCode::NOT_FOUND);
            }
            _ => {
                tracing::error!("failed to update block: {:?}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
    };

    Ok((StatusCode::OK, Json(block)))
}

#[axum::debug_handler]
#[instrument(skip(state))]
pub async fn delete_resource(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let mut db_blocks = match state.db().blocks().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if let Err(err) = db_blocks.delete(&id).await {
        match err.downcast_ref::<DbError>() {
            Some(DbError::NotFound) => {
                tracing::error!("block could not be found: {:?}", err);
                return Err(StatusCode::NOT_FOUND);
            }
            _ => {
                tracing::error!("failed to delete block: {:?}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    };

    Ok(StatusCode::OK)
}
