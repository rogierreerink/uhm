use crate::global::AppState;

use axum::Router;
use std::sync::Arc;

mod collection;
mod resource;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(collection::create_router(state.clone()))
        .merge(resource::create_router(state))
}
