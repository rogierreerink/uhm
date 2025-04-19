use crate::db::list_items::{DbListItems, ListItem, ListItemDataNew, ListItemMinimal};
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
    data: Vec<ListItem>,
}

#[axum::debug_handler]
#[instrument(skip(state))]
pub async fn get_collection(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut db_list_items = match state.db().list_items().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let list_items = match db_list_items.get().await {
        Ok(list_items) => list_items,
        Err(err) => {
            tracing::error!("failed to get list items: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((StatusCode::OK, Json(GetResponse { data: list_items })))
}

#[derive(Deserialize)]
struct PostRequest {
    data: Vec<ListItemDataNew>,
}

#[derive(Serialize)]
struct PostResponse {
    data: Vec<ListItemMinimal>,
}

#[axum::debug_handler]
#[instrument(skip(state, list_item))]
pub async fn post_collection(
    State(state): State<Arc<AppState>>,
    Json(list_item): Json<PostRequest>,
) -> impl IntoResponse {
    let mut db_list_items = match state.db().list_items().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let list_items = match db_list_items.create(&list_item.data).await {
        Ok(list_items) => list_items,
        Err(err) => {
            tracing::error!("failed to create list items: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((StatusCode::CREATED, Json(PostResponse { data: list_items })))
}
