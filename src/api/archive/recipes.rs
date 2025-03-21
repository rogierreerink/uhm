use std::sync::Arc;

use axum::{
    http::{header, HeaderValue},
    routing::{get, options},
    Router,
};
use tower::ServiceBuilder;
use tower_http::set_header::SetResponseHeaderLayer;
use tracing::instrument;

use crate::global::AppState;

mod ingredients;
mod recipe_get;
mod recipes_get;

#[instrument(skip(state))]
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(
            Router::new()
                .route("/", get(recipes_get::handle))
                // .route("/", post(recipes_post::handle))
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
                .route("/:slug", get(recipe_get::handle))
                // .route("/:slug", patch(recipe_patch::handle))
                // .route("/:slug", delete(recipe_delete::handle))
                .route("/:slug", options(handle_options))
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
        .with_state(state.clone())
        .nest("/:slug/ingredients", ingredients::create_router(state))
}

#[instrument()]
async fn handle_options() {}
