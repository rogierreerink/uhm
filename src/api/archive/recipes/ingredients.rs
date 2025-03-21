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

mod ingredient_delete;
mod ingredient_get;
mod ingredient_patch;
mod ingredients_get;
mod ingredients_post;

#[instrument(skip(state))]
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(
            Router::new()
                .route("/", get(ingredients_get::handle))
                .route("/", post(ingredients_post::handle))
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
                .route("/:id", get(ingredient_get::handle))
                .route("/:id", patch(ingredient_patch::handle))
                .route("/:id", delete(ingredient_delete::handle))
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
