use axum::{
    extract::{Path, Query},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use starknet_types::{Call, ChainId};
use std::collections::HashMap;
use std::str::FromStr;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing_subscriber::{self, EnvFilter};

#[derive(Serialize, Deserialize, Debug)]
struct RouteParams {
    method: String,
    network: String,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Build our application with routes
    let app = Router::new()
        .route("/", get(index))
        .route("/deposit/:method/:network/", get(handle_deposit))
        .nest_service(
            "/static",
            ServeDir::new("crates/bins/website-server/static"),
        )
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()));

    // Run it with hyper on localhost:3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("🚀 Server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> impl IntoResponse {
    Html(include_str!("../templates/index.html"))
}

async fn handle_deposit(
    Path(params): Path<RouteParams>,
    Query(query_params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // Validate method parameter
    if params.method != "starknet" {
        return Html(format!(
            r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Invalid Method</title>
                <link rel="stylesheet" href="/static/styles.css">
            </head>
            <body>
                <div class="container">
                    <header>
                        <h1>❌ Invalid Method</h1>
                        <p>Only 'starknet' method is supported</p>
                    </header>
                    <main>
                        <div class="info-section">
                            <p>Received method: <code>{}</code></p>
                            <p>Allowed method: <code>starknet</code></p>
                            <p><a href="/">Go back to home</a></p>
                        </div>
                    </main>
                </div>
            </body>
            </html>
            "#,
            params.method
        ));
    }

    // Validate network parameter using ChainId
    let chain_id = match ChainId::from_str(&params.network) {
        Ok(ChainId::Mainnet) | Ok(ChainId::Sepolia) | Ok(ChainId::Devnet) => {
            ChainId::from_str(&params.network).unwrap()
        }
        Ok(ChainId::Custom(_)) | Err(_) => {
            return Html(format!(
                r#"
                <!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>Invalid Network</title>
                    <link rel="stylesheet" href="/static/styles.css">
                </head>
                <body>
                    <div class="container">
                        <header>
                            <h1>❌ Invalid Network</h1>
                            <p>Unsupported network specified</p>
                        </header>
                        <main>
                            <div class="info-section">
                                <p>Received network: <code>{}</code></p>
                                <p>Allowed networks: <code>SN_SEPOLIA</code>, <code>SN_DEVNET</code>, <code>SN_MAINNET</code></p>
                                <p><a href="/">Go back to home</a></p>
                            </div>
                        </main>
                    </div>
                </body>
                </html>
                "#,
                params.network
            ));
        }
    };

    let calldata_raw = query_params
        .get("calldata")
        .unwrap_or(&String::new())
        .clone();

    // Parse calldata as Vec<Call> - return error if it fails
    let calls = match serde_json::from_str::<Vec<Call>>(&calldata_raw) {
        Ok(calls) => calls,
        Err(err) => {
            return Html(format!(
                r#"
                <!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>Invalid Calldata</title>
                    <link rel="stylesheet" href="/static/styles.css">
                </head>
                <body>
                    <div class="container">
                        <header>
                            <h1>❌ Invalid Calldata</h1>
                            <p>Calldata must be deserializable as Vec&lt;Call&gt;</p>
                        </header>
                        <main>
                            <div class="info-section">
                                <div class="info-item">
                                    <label>Error:</label>
                                    <span class="value">{}</span>
                                </div>
                                <div class="info-item calldata-section">
                                    <label>Received Calldata:</label>
                                    <pre class="calldata-value">{}</pre>
                                </div>
                                <p><a href="/">Go back to home</a></p>
                            </div>
                        </main>
                    </div>
                </body>
                </html>
                "#,
                err, calldata_raw
            ));
        }
    };

    let formatted_calldata = serde_json::to_string_pretty(&calls).unwrap_or(calldata_raw.clone());
    let calls_info = format!(
        "<div class=\"info-item\"><label>Parsed Calls:</label><span class=\"value\">{} call(s)</span></div>",
        calls.len()
    );

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Deposit Route Handler</title>
            <link rel="stylesheet" href="/static/styles.css">
        </head>
        <body>
            <div class="container">
                <header>
                    <h1>🔗 Route Handler</h1>
                    <p>Starknet Deposit endpoint</p>
                </header>

                <main>
                    <div class="info-section">
                        <div class="info-item">
                            <label>Method:</label>
                            <span class="value">{}</span>
                        </div>
                        
                        <div class="info-item">
                            <label>Network:</label>
                            <span class="value">{}</span>
                        </div>

                        <div class="info-item">
                            <label>Chain ID:</label>
                            <span class="value">{}</span>
                        </div>
                        
                        {}
                        
                        <div class="info-item calldata-section">
                            <label>Calldata:</label>
                            <pre class="calldata-value">{}</pre>
                        </div>
                    </div>
                </main>

                <footer>
                    <p>Built with Rust + Axum + Starknet</p>
                </footer>
            </div>
        </body>
        </html>
        "#,
        params.method, params.network, chain_id, calls_info, formatted_calldata
    );

    Html(html)
}
