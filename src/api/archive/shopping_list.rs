use axum::{
    http::{header, HeaderValue},
    routing::{delete, get, options, patch, post},
    Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::set_header::SetResponseHeaderLayer;
use tracing::instrument;

use crate::global::AppState;

mod shopping_list_item_delete;
mod shopping_list_item_get;
mod shopping_list_item_patch;
mod shopping_list_items_get;
mod shopping_list_items_post;

#[instrument(skip(state))]
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(
            Router::new()
                .route("/", get(shopping_list_items_get::handle))
                .route("/", post(shopping_list_items_post::handle))
                .route("/", options(handle_options))
                .layer(
                    ServiceBuilder::new()
                        .layer(SetResponseHeaderLayer::if_not_present(
                            header::ACCESS_CONTROL_ALLOW_METHODS,
                            HeaderValue::from_static("GET, POST"),
                        ))
                        .layer(SetResponseHeaderLayer::if_not_present(
                            header::ACCESS_CONTROL_ALLOW_HEADERS,
                            HeaderValue::from_static("content-type"),
                        )),
                )
                .with_state(state.clone()),
        )
        .merge(
            Router::new()
                .route("/:id", get(shopping_list_item_get::handle))
                .route("/:id", patch(shopping_list_item_patch::handle))
                .route("/:id", delete(shopping_list_item_delete::handle))
                .route("/:id", options(handle_options))
                .layer(
                    ServiceBuilder::new()
                        .layer(SetResponseHeaderLayer::if_not_present(
                            header::ACCESS_CONTROL_ALLOW_METHODS,
                            HeaderValue::from_static("GET, PATCH, DELETE"),
                        ))
                        .layer(SetResponseHeaderLayer::if_not_present(
                            header::ACCESS_CONTROL_ALLOW_HEADERS,
                            HeaderValue::from_static("content-type"),
                        )),
                )
                .with_state(state.clone()),
        )
}

#[instrument()]
async fn handle_options() {}
