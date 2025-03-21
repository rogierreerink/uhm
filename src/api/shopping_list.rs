use crate::api::handle_options;
use crate::global::AppState;

use axum::{
    routing::{delete, get, options, patch, post},
    Router,
};
use std::sync::Arc;

mod collection_get;
mod collection_post;
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
                .with_state(state.clone()),
        )
        .merge(
            Router::new()
                .route("/:id", get(resource_get::handle))
                .route("/:id", patch(resource_patch::handle))
                .route("/:id", delete(resource_delete::handle))
                .route("/:id", options(handle_options))
                .with_state(state.clone()),
        )
}
