use crate::global::AppState;
use axum::Router;
use std::sync::Arc;

mod blocks;
mod ingredient_collections;
mod list_items;
mod lists;
mod markdown;
mod pages;
mod products;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/blocks", blocks::create_router(state.clone()))
        .nest(
            "/ingredient-collections",
            ingredient_collections::create_router(state.clone()),
        )
        .nest("/list-items", list_items::create_router(state.clone()))
        .nest("/lists", lists::create_router(state.clone()))
        .nest("/markdown", markdown::create_router(state.clone()))
        .nest("/pages", pages::create_router(state.clone()))
        .nest("/products", products::create_router(state.clone()))
}

pub async fn handle_options() {}
