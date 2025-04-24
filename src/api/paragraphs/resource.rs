use crate::api::handle_options;
use crate::db::paragraphs::{ParagraphDb, ParagraphUpdate};
use crate::db::{Db, DbError};
use crate::global::AppState;

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
    let mut db = match state.db().paragraphs().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let item = match db.get_by_id(&id).await {
        Ok(block) => block,
        Err(err) => match err.downcast_ref::<DbError>() {
            Some(DbError::NotFound) => {
                tracing::error!("item could not be found: {:?}", err);
                return Err(StatusCode::NOT_FOUND);
            }
            _ => {
                tracing::error!("failed to get item: {:?}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
    };

    Ok((StatusCode::OK, Json(item)))
}

#[axum::debug_handler]
#[instrument(skip(state, payload))]
pub async fn patch_resource(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ParagraphUpdate>,
) -> impl IntoResponse {
    let mut db = match state.db().paragraphs().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let updated = match db.update_by_id(&id, payload).await {
        Ok(updated) => updated,
        Err(err) => match err.downcast_ref::<DbError>() {
            Some(DbError::NotFound) => {
                tracing::error!("item could not be found: {:?}", err);
                return Err(StatusCode::NOT_FOUND);
            }
            _ => {
                tracing::error!("failed to update item: {:?}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
    };

    Ok((StatusCode::OK, Json(updated)))
}

#[axum::debug_handler]
#[instrument(skip(state))]
pub async fn delete_resource(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let mut db = match state.db().paragraphs().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if let Err(err) = db.delete_by_id(&id).await {
        match err.downcast_ref::<DbError>() {
            Some(DbError::NotFound) => {
                tracing::error!("item could not be found: {:?}", err);
                return Err(StatusCode::NOT_FOUND);
            }
            _ => {
                tracing::error!("failed to delete item: {:?}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    };

    Ok(StatusCode::OK)
}
