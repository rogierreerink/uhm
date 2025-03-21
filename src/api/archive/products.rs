use std::sync::Arc;

use axum::{
    http::{header, HeaderValue},
    routing::{delete, get, options, patch, post},
    Router,
};
use tower::ServiceBuilder;
use tower_http::set_header::SetResponseHeaderLayer;
use tracing::instrument;

use crate::global::AppState;

mod product_delete;
mod product_get;
mod product_patch;
mod products_get;
mod products_post;

#[instrument(skip(state))]
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(
            Router::new()
                .route("/", get(products_get::handle))
                .route("/", post(products_post::handle))
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
                ),
        )
        .merge(
            Router::new()
                .route("/:id", get(product_get::handle))
                .route("/:id", patch(product_patch::handle))
                .route("/:id", delete(product_delete::handle))
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
                ),
        )
        .with_state(state)
}

#[instrument()]
async fn handle_options() {}
