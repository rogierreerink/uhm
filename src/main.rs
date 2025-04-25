use std::{env, sync::Arc};

use axum::{
    http::{header, HeaderValue},
    Router,
};
use db::DbPostgres;
use global::{AppDb, AppState};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceBuilder;
use tower_http::set_header::SetResponseHeaderLayer;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod db;
mod global;
mod utilities;

#[tokio::main]
async fn main() {
    let tracing_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::registry()
        .with(tracing_filter)
        .with(fmt::layer())
        .init();

    tracing::info!("starting application");

    tracing::info!("creating database pool");
    let db_pool = match PgPoolOptions::new()
        .connect(&format!(
            "postgres://{}:{}@{}:{}/{}",
            env::var("DB_USER").unwrap_or("postgres".into()),
            env::var("DB_PASSWORD").unwrap_or("postgres".into()),
            env::var("DB_HOST").unwrap_or("localhost".into()),
            env::var("DB_PORT").unwrap_or("5432".into()),
            env::var("DB_NAME").unwrap_or("postgres".into())
        ))
        .await
    {
        Ok(pool) => pool,
        Err(error) => {
            tracing::error!("failed create database pool: {}", error);
            return;
        }
    };

    let app_state = Arc::new(AppState {
        db: AppDb::Postgres(DbPostgres::new(db_pool)),
    });

    tracing::info!("setting up routes");
    let app = Router::new()
        .nest("/api", api::create_router(app_state))
        .layer(
            ServiceBuilder::new().layer(SetResponseHeaderLayer::if_not_present(
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                HeaderValue::from_static("*"),
            )),
        );

    let ip = env::var("LISTENER_IP").unwrap_or("0.0.0.0".into());
    let port = env::var("LISTENER_PORT").unwrap_or("3002".into());
    let address = format!("{}:{}", ip, port);

    tracing::info!("binding to address {}", address);
    let listener = match tokio::net::TcpListener::bind(address).await {
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
