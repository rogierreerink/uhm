use std::sync::Arc;

use axum::{routing::get, Router};
use tracing::instrument;

use crate::global::AppState;

mod collection_get;
mod collections_get;

#[instrument(skip(state))]
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(collections_get::handle))
        .route("/:slug", get(collection_get::handle))
        .with_state(state)
}
