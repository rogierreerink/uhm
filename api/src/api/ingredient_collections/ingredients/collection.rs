use crate::db::ingredients::{IngredientCreate, IngredientDb};
use crate::global::AppState;
use crate::utilities::request::collection::{GetResponse, PostRequest, PostResponse};
use crate::{api::handle_options, db::Db};

use axum::extract::Path;
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
use uuid::Uuid;

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
    Path(collection_id): Path<Uuid>,
) -> impl IntoResponse {
    let mut db = state.db().ingredients();

    let items = match db.get_multiple(&collection_id).await {
        Ok(items) => items,
        Err(err) => {
            tracing::error!("failed to get items: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((
        StatusCode::OK,
        Json(GetResponse {
            pagination: None,
            data: items,
        }),
    ))
}

#[axum::debug_handler]
#[instrument(skip(state, payload))]
pub async fn post_collection(
    State(state): State<Arc<AppState>>,
    Path(collection_id): Path<Uuid>,
    Json(payload): Json<PostRequest<IngredientCreate>>,
) -> impl IntoResponse {
    let mut db = state.db().ingredients();

    let created = match db.create_multiple(&collection_id, payload.data).await {
        Ok(created) => created,
        Err(err) => {
            tracing::error!("failed to create items: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((StatusCode::CREATED, Json(PostResponse { data: created })))
}
