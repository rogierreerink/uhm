use crate::db::ingredient_collections::{
    DbIngredientCollections, IngredientCollectionDataNew, IngredientCollectionMinimal,
};
use crate::global::AppState;
use crate::{api::handle_options, db::Db};

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

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new().merge(
        Router::new()
            .route("/", get(get_collection))
            .route("/", post(post_collection))
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
    data: Vec<IngredientCollectionMinimal>,
}

#[axum::debug_handler]
#[instrument(skip(state))]
pub async fn get_collection(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut db_ingredient_collections = match state.db().ingredient_collections().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let collections = match db_ingredient_collections.get().await {
        Ok(collections) => collections,
        Err(err) => {
            tracing::error!("failed to get ingredient collections: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((StatusCode::OK, Json(GetResponse { data: collections })))
}

#[derive(Deserialize)]
struct PostRequest {
    data: Vec<IngredientCollectionDataNew>,
}

#[derive(Serialize)]
struct PostResponse {
    data: Vec<IngredientCollectionMinimal>,
}

#[axum::debug_handler]
#[instrument(skip(state, ingredient_collection))]
pub async fn post_collection(
    State(state): State<Arc<AppState>>,
    Json(ingredient_collection): Json<PostRequest>,
) -> impl IntoResponse {
    let mut db_ingredient_collections = match state.db().ingredient_collections().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let collections = match db_ingredient_collections
        .create(&ingredient_collection.data)
        .await
    {
        Ok(collections) => collections,
        Err(err) => {
            tracing::error!("failed to create ingredient collections: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((
        StatusCode::CREATED,
        Json(PostResponse { data: collections }),
    ))
}
