use crate::db::products::{DbProducts, Product, ProductDataNew, ProductMinimal, QueryParams};
use crate::global::AppState;
use crate::{api::handle_options, db::Db};

use axum::extract::Query;
use axum::{
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, options, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::set_header::SetResponseHeaderLayer;
use tracing::instrument;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new().merge(
        Router::new()
            .route("/", get(get_collection))
            .route("/", post(post_collection))
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
}

#[derive(Serialize)]
struct GetResponse {
    data: Vec<Product>,
}

#[axum::debug_handler]
#[instrument(skip(state))]
pub async fn get_collection(
    State(state): State<Arc<AppState>>,
    Query(search_query): Query<QueryParams>,
) -> impl IntoResponse {
    let mut db_products = match state.db().products().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let products = match db_products.get(&search_query).await {
        Ok(products) => products,
        Err(err) => {
            tracing::error!("failed to get products: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((StatusCode::OK, Json(GetResponse { data: products })))
}

#[derive(Deserialize)]
struct PostRequest {
    data: Vec<ProductDataNew>,
}

#[derive(Serialize)]
struct PostResponse {
    data: Vec<ProductMinimal>,
}

#[axum::debug_handler]
#[instrument(skip(state, product))]
pub async fn post_collection(
    State(state): State<Arc<AppState>>,
    Json(product): Json<PostRequest>,
) -> impl IntoResponse {
    let mut db_products = match state.db().products().await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("failed to connect to database: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let products = match db_products.create(&product.data).await {
        Ok(products) => products,
        Err(err) => {
            tracing::error!("failed to create products: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((StatusCode::CREATED, Json(PostResponse { data: products })))
}
