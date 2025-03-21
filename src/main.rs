use std::sync::Arc;

use axum::{
    http::{header, HeaderValue},
    Router,
};
use global::AppState;
use tower::ServiceBuilder;
use tower_http::set_header::SetResponseHeaderLayer;
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod db;
mod global;
mod types;
mod utilities;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    tracing::info!("starting application");

    let mut db_config = deadpool_postgres::Config::new();
    db_config.user = Some("postgres".into());
    db_config.password = Some("postgres".into());
    db_config.dbname = Some("postgres".into());
    // db_config.host = Some("localhost".into());
    db_config.host = Some("host.docker.internal".into());
    db_config.port = Some(5432);

    tracing::info!("creating database pool");
    let db_pool =
        match db_config.create_pool(Some(deadpool::Runtime::Tokio1), tokio_postgres::NoTls) {
            Ok(pool) => pool,
            Err(err) => {
                tracing::error!("failed create database pool: {}", err);
                return;
            }
        };

    let app_state = Arc::new(AppState { db_pool });

    tracing::info!("setting up routes");
    let app = Router::new()
        .nest("/api", api::create_router(app_state))
        .layer(
            ServiceBuilder::new().layer(SetResponseHeaderLayer::if_not_present(
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                HeaderValue::from_static("*"),
            )),
        );

    tracing::info!("binding to port {}", 3002);
    let listener = match tokio::net::TcpListener::bind("0.0.0.0:3002").await {
        Ok(listener) => listener,
        Err(err) => {
            tracing::error!("failed to bind to port: {}", err);
            return;
        }
    };

    tracing::info!("starting server");
    if let Err(err) = axum::serve(listener, app).await {
        tracing::error!("failed to start server: {}", err);
        return;
    }
}
