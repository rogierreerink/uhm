use crate::global::AppState;
use axum::Router;
use serde::Serialize;
use std::sync::Arc;

mod blocks;
mod ingredient_collections;
mod list_items;
mod markdown;
mod products;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/blocks", blocks::create_router(state.clone()))
        .nest(
            "/ingredient-collections",
            ingredient_collections::create_router(state.clone()),
        )
        .nest("/list-items", list_items::create_router(state.clone()))
        .nest("/markdown", markdown::create_router(state.clone()))
        .nest("/products", products::create_router(state.clone()))
}

pub async fn handle_options() {}

#[derive(Serialize)]
pub struct Pagination {
    pub skip: usize,
    pub take: usize,
    pub total: usize,
}
