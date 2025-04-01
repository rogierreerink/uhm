use crate::global::AppState;
use axum::Router;
use serde::Serialize;
use std::sync::Arc;

mod blocks;
mod ingredient_collections;
mod products;
// mod shopping_list;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/blocks", blocks::create_router(state.clone()))
        .nest("/products", products::create_router(state.clone()))
        // .nest(
        //     "/shopping-list",
        //     shopping_list::create_router(state.clone()),
        // )
        .nest(
            "/ingredient-collections",
            ingredient_collections::create_router(state),
        )
}

pub async fn handle_options() {}

#[derive(Serialize)]
pub struct Pagination {
    pub skip: usize,
    pub take: usize,
    pub total: usize,
}
