use abis::{IERC20_CONTRACT_ABI, INVOICE_CONTRACT_ABI};
use askama::Template;
use axum::{
    extract::{Path, Query},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use starknet_core::types::{contract::AbiEntry, Felt};
use starknet_types::{constants::ON_CHAIN_CONSTANTS, ChainId, PayInvoiceCallData};
use std::str::FromStr;
use std::{collections::HashMap, net::SocketAddr};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing_subscriber::{self, EnvFilter};

mod abis;
mod serve;

use serve::serve;

#[derive(Serialize, Deserialize, Debug)]
struct RouteParams {
    method: String,
    network: String,
}

#[derive(Template)]
#[template(path = "invalid_method.html")]
struct InvalidMethodTemplate {
    method: String,
}

#[derive(Template)]
#[template(path = "invalid_network.html")]
struct InvalidNetworkTemplate {
    network: String,
}

#[derive(Template)]
#[template(path = "invalid_payload.html")]
struct InvalidPayloadTemplate {
    error: String,
    payload_raw: String,
}

#[derive(Template)]
#[template(path = "deposit.html")]
struct DepositTemplate {
    method: String,
    network: String,
    formatted_payload: String,
    deposit_data: DepositData,
}

#[derive(Template)]
#[template(path = "salto_landing.html")]
struct SaltoLandingTemplate {}

#[derive(Debug, Serialize, Deserialize)]
struct ConctractData {
    abi: Vec<AbiEntry>,
    address: Felt,
}

#[derive(Debug, Serialize, Deserialize)]
struct DepositData {
    provider_url: String,
    invoice_contract: ConctractData,
    asset_contract: ConctractData,
    quote_id_hash: Felt,
    expiry: Felt,
    amount_low: Felt,
    amount_high: Felt,
    payee: Felt,
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
        .route("/salto", get(salto_landing))
        .route("/deposit/{method}/{network}/", get(handle_deposit))
        .nest_service("/static", ServeDir::new("crates/bins/web-app/static"))
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()));

    // Get port from environment variable or use default
    let port = std::env::var("PORT").unwrap_or_else(|_| "443".to_string());
    let bind_address: SocketAddr = format!("0.0.0.0:{}", port)
        .parse()
        .expect("Invalid bind address");

    serve(app, bind_address).await;
}

async fn index() -> impl IntoResponse {
    Html(include_str!("../templates/index.html"))
}

async fn salto_landing() -> impl IntoResponse {
    let template = SaltoLandingTemplate {};
    Html(
        template
            .render()
            .unwrap_or_else(|_| "Template render error".to_string()),
    )
}

async fn handle_deposit(
    Path(params): Path<RouteParams>,
    Query(query_params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // Validate method parameter
    if params.method != "starknet" {
        let template = InvalidMethodTemplate {
            method: params.method,
        };
        return Html(
            template
                .render()
                .unwrap_or_else(|_| "Template render error".to_string()),
        );
    }

    // Validate network parameter using ChainId
    let chain_id = match ChainId::from_str(&params.network) {
        Ok(ChainId::Custom(_)) | Ok(ChainId::Mainnet) | Err(_) => {
            let template = InvalidNetworkTemplate {
                network: params.network,
            };
            return Html(
                template
                    .render()
                    .unwrap_or_else(|_| "Template render error".to_string()),
            );
        }
        Ok(ChainId::Sepolia) => ChainId::Sepolia,
        Ok(ChainId::Devnet) => ChainId::Devnet,
    };

    let payload_raw = query_params
        .get("payload")
        .unwrap_or(&String::new())
        .clone();

    let pay_invoice_call_data = match serde_json::from_str::<PayInvoiceCallData>(&payload_raw) {
        Ok(payload) => payload,
        Err(err) => {
            let template = InvalidPayloadTemplate {
                error: err.to_string(),
                payload_raw,
            };
            return Html(
                template
                    .render()
                    .unwrap_or_else(|_| "Template render error".to_string()),
            );
        }
    };

    let formatted_payload =
        serde_json::to_string_pretty(&pay_invoice_call_data).unwrap_or(payload_raw.clone());

    let on_chain_constants = ON_CHAIN_CONSTANTS
        .get(chain_id.as_str())
        .expect("a supported chain");

    let provider_url = match &chain_id {
        ChainId::Devnet => "http://localhost:5050".to_string(),
        ChainId::Sepolia => "https://starknet-sepolia.public.blastapi.io/rpc/v0_8".to_string(),
        ChainId::Custom(_) | ChainId::Mainnet => panic!("unsuported at the moment"),
    };

    let deposit_data = DepositData {
        provider_url,
        invoice_contract: ConctractData {
            abi: INVOICE_CONTRACT_ABI.clone(),
            address: on_chain_constants.invoice_payment_contract_address,
        },
        asset_contract: ConctractData {
            abi: vec![IERC20_CONTRACT_ABI.clone()],
            address: pay_invoice_call_data.asset_contract_address,
        },
        quote_id_hash: pay_invoice_call_data.quote_id_hash,
        expiry: pay_invoice_call_data.expiry,
        amount_low: pay_invoice_call_data.amount.low,
        amount_high: pay_invoice_call_data.amount.high,
        payee: pay_invoice_call_data.payee,
    };

    let template = DepositTemplate {
        method: params.method,
        network: params.network,
        formatted_payload,
        deposit_data,
    };

    Html(
        template
            .render()
            .unwrap_or_else(|_| "Template render error".to_string()),
    )
}
