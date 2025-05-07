use crate::db::products::{ProductCreate, ProductDb, SearchParams};
use crate::global::AppState;
use crate::utilities::request::collection::{GetResponse, PostRequest, PostResponse};
use crate::{api::handle_options, db::Db};

use axum::extract::Query;
use axum::{
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, options, post},
    Json, Router,
};
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

#[axum::debug_handler]
#[instrument(skip(state))]
pub async fn get_collection(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchParams>,
) -> impl IntoResponse {
    let mut db = state.db().products();

    let items = match db.get_multiple(query.clone()).await {
        Ok(items) => items,
        Err(err) => {
            tracing::error!("failed to get items: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((
        StatusCode::OK,
        Json(GetResponse {
            pagination: Some(items.1),
            data: items.0,
        }),
    ))
}

#[axum::debug_handler]
#[instrument(skip(state, payload))]
pub async fn post_collection(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PostRequest<ProductCreate>>,
) -> impl IntoResponse {
    let mut db = state.db().products();

    let created = match db.create_multiple(payload.data).await {
        Ok(created) => created,
        Err(err) => {
            tracing::error!("failed to create items: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((StatusCode::CREATED, Json(PostResponse { data: created })))
}
