use std::sync::Arc;

use axum::{routing::get, Router};
use tracing::instrument;

use crate::global::AppState;

mod unit_get;
mod units_get;

#[instrument(skip(state))]
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(units_get::handle))
        .route("/:id", get(unit_get::handle))
        .with_state(state)
}
