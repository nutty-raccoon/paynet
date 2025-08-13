use crate::{app_state::AppState, rest_service};
use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

pub async fn launch_rest_server_task(
    app_state: AppState,
    rest_port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = create_rest_app(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], rest_port));
    let listener = TcpListener::bind(addr).await?;

    info!("HTTP REST API server listening on {}", addr);

    axum::serve(listener, app).await.map_err(|e| {
        warn!("HTTP server error: {}", e);
        e
    })?;

    Ok(())
}

fn create_rest_app(app_state: AppState) -> Router {
    rest_service::create_router()
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::OPTIONS,
                ])
                .allow_headers(tower_http::cors::Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}
