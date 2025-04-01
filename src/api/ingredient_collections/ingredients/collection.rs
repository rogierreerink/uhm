use crate::db::ingredients::{DbIngredients, IngredientMinimal, IngredientNew, IngredientSummary};
use crate::global::AppState;
use crate::{api::handle_options, db::Db};

use axum::extract::Path;
use axum::{
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, options, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::set_header::SetResponseHeaderLayer;
use tracing::instrument;
use uuid::Uuid;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new().merge(
        Router::new()
            .route("/", get(get_ingredients))
            .route("/", post(post_ingredients))
            .route("/", options(handle_options))
            .layer(
                ServiceBuilder::new()
                    .layer(SetResponseHeaderLayer::if_not_present(
                        header::ACCESS_CONTROL_ALLOW_METHODS,
                        HeaderValue::from_static("GET, POST, OPTIONS"),
                    ))
                    .layer(SetResponseHeaderLayer::if_not_present(
                        header::ACCESS_CONTROL_ALLOW_HEADERS,
                        HeaderValue::from_static("content-type"),
                    )),
            )
            .with_state(state.clone()),
    )
}

#[derive(Serialize)]
struct GetResponse {
    data: Vec<IngredientSummary>,
}

#[axum::debug_handler]
#[instrument(skip(state))]
pub async fn get_ingredients(
    State(state): State<Arc<AppState>>,
    Path(collection_id): Path<Uuid>,
) -> impl IntoResponse {
    let mut db_ingredients = match state.db().ingredients().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let ingredients = match db_ingredients.get(&collection_id).await {
        Ok(ingredients) => ingredients,
        Err(err) => {
            tracing::error!("failed to get ingredients: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((StatusCode::OK, Json(GetResponse { data: ingredients })))
}

#[derive(Deserialize)]
struct PostRequest {
    data: Vec<IngredientNew>,
}

#[derive(Serialize)]
struct PostResponse {
    data: Vec<IngredientMinimal>,
}

#[axum::debug_handler]
#[instrument(skip(state, ingredients))]
pub async fn post_ingredients(
    State(state): State<Arc<AppState>>,
    Path(collection_id): Path<Uuid>,
    Json(ingredients): Json<PostRequest>,
) -> impl IntoResponse {
    let mut db_ingredients = match state.db().ingredients().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let ingredients = match db_ingredients
        .create(&collection_id, &ingredients.data)
        .await
    {
        Ok(ingredients) => ingredients,
        Err(err) => {
            tracing::error!("failed to create ingredients: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((
        StatusCode::CREATED,
        Json(PostResponse { data: ingredients }),
    ))
}
