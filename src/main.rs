mod config;
mod errors;
mod handlers;
mod models;
mod services;
mod state;
mod utils;

use std::sync::Arc;

use axum::routing::{get, post};
use socketioxide::SocketIo;
use tower_http::cors::{AllowHeaders, AllowMethods, CorsLayer};
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::handlers::{http, socket};
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env();
    let app_state = Arc::new(AppState::default());

    let (layer, io) = SocketIo::builder()
        .with_state(app_state.clone())
        .build_layer();

    io.ns("/", socket::on_connect);

    let origins: Vec<axum::http::HeaderValue> = config
        .cors_origins
        .iter()
        .map(|o| o.parse().expect("Invalid CORS origin"))
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(AllowMethods::any())
        .allow_headers(AllowHeaders::any());

    let app = axum::Router::new()
        .route("/health", get(http::health))
        .route("/api/rooms", post(http::create_room))
        .route("/api/rooms/{room_id}", get(http::get_room))
        .layer(layer)
        .layer(cors)
        .with_state(app_state);

    let addr = format!("0.0.0.0:{}", config.port);
    info!("Pockerra backend starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
