use crate::db::blocks::{Block, BlockNew, BlockSummary, DbBlocks};
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
    data: Vec<BlockSummary>,
}

#[axum::debug_handler]
#[instrument(skip(state))]
pub async fn get_collection(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut db_blocks = match state.db().blocks().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let blocks = match db_blocks.get().await {
        Ok(blocks) => blocks,
        Err(err) => {
            tracing::error!("failed to get blocks: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((StatusCode::OK, Json(GetResponse { data: blocks })))
}

#[derive(Deserialize)]
struct PostRequest {
    data: Vec<BlockNew>,
}

#[derive(Serialize)]
struct PostResponse {
    data: Vec<Block>,
}

#[axum::debug_handler]
#[instrument(skip(state, block))]
pub async fn post_collection(
    State(state): State<Arc<AppState>>,
    Json(block): Json<PostRequest>,
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

    Ok((StatusCode::CREATED, Json(PostResponse { data: blocks })))
}
