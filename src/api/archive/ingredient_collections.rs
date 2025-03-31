use crate::api::handle_options;
use crate::global::AppState;

use axum::{
    http::{header, HeaderValue},
    routing::{delete, get, options, patch, post},
    Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::set_header::SetResponseHeaderLayer;

mod collection_get;
mod collection_post;
mod ingredients;
mod resource_delete;
mod resource_get;
mod resource_patch;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(
            Router::new()
                .route("/", get(collection_get::handle))
                .route("/", post(collection_post::handle))
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
        .merge(
            Router::new()
                .route("/:id", get(resource_get::handle))
                .route("/:id", patch(resource_patch::handle))
                .route("/:id", delete(resource_delete::handle))
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
                .with_state(state.clone()),
        )
        .nest(
            "/:id/ingredients",
            ingredients::create_router(state.clone()),
        )
}
