use crate::global::AppState;
use axum::Router;
use std::sync::Arc;

mod products;
mod shopping_list;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/products", products::create_router(state.clone()))
        .nest("/shopping-list", shopping_list::create_router(state))
}

pub async fn handle_options() {}
