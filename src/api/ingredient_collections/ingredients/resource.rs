use crate::db::ingredients::{DbIngredients, IngredientUpdate};
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
    Path((collection_id, id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let mut db_ingredients = match state.db().ingredients().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let ingredient = match db_ingredients.get_by_id(&collection_id, &id).await {
        Ok(ingredient) => ingredient,
        Err(err) => match err.downcast_ref::<DbError>() {
            Some(DbError::NotFound) => {
                tracing::error!("ingredient could not be found: {:?}", err);
                return Err(StatusCode::NOT_FOUND);
            }
            _ => {
                tracing::error!("failed to get ingredient: {:?}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
    };

    Ok((StatusCode::OK, Json(ingredient)))
}

#[axum::debug_handler]
#[instrument(skip(state, ingredient))]
pub async fn patch_resource(
    State(state): State<Arc<AppState>>,
    Path((collection_id, id)): Path<(Uuid, Uuid)>,
    Json(ingredient): Json<IngredientUpdate>,
) -> impl IntoResponse {
    let mut db_ingredients = match state.db().ingredients().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let ingredient = match db_ingredients
        .update(&collection_id, &id, &ingredient)
        .await
    {
        Ok(ingredient) => ingredient,
        Err(err) => match err.downcast_ref::<DbError>() {
            Some(DbError::NotFound) => {
                tracing::error!("ingredient could not be found: {:?}", err);
                return Err(StatusCode::NOT_FOUND);
            }
            _ => {
                tracing::error!("failed to update ingredient: {:?}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
    };

    Ok((StatusCode::OK, Json(ingredient)))
}

#[axum::debug_handler]
#[instrument(skip(state))]
pub async fn delete_resource(
    State(state): State<Arc<AppState>>,
    Path((collection_id, id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let mut db_ingredients = match state.db().ingredients().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if let Err(err) = db_ingredients.delete(&collection_id, &id).await {
        match err.downcast_ref::<DbError>() {
            Some(DbError::NotFound) => {
                tracing::error!("ingredient could not be found: {:?}", err);
                return Err(StatusCode::NOT_FOUND);
            }
            _ => {
                tracing::error!("failed to delete ingredient: {:?}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    };

    Ok(StatusCode::OK)
}
