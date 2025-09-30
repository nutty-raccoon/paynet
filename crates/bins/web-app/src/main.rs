use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower::{Layer, ServiceBuilder};
use tower_http::{cors::CorsLayer, normalize_path::NormalizePathLayer, services::ServeDir};
use tracing_subscriber::{self, EnvFilter};

mod abis;
mod routes;
mod serve;
mod types;

use routes::{deposit_landing, handle_deposit, health_check, salto_landing};
use serve::serve;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/salto", get(salto_landing))
        .route("/deposit", get(deposit_landing))
        .route("/deposit/{method}/{network}", get(handle_deposit))
        .nest_service("/static", ServeDir::new("crates/bins/web-app/static"))
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()));
    let app = NormalizePathLayer::trim_trailing_slash().layer(app);

    // Get port from environment variable or use default
    let port = std::env::var("PORT").unwrap_or_else(|_| "443".to_string());
    let bind_address: SocketAddr = format!("0.0.0.0:{}", port)
        .parse()
        .expect("Invalid bind address");

    serve(app, bind_address).await;
}
