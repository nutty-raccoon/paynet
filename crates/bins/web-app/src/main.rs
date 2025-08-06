use axum::{
    extract::{Path, Query},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use starknet_core::types::{contract::AbiEntry, Felt};
use starknet_types::{constants::ON_CHAIN_CONSTANTS, ChainId, DepositPayload};
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
        .nest_service("/static", ServeDir::new("crates/bins/web-app/static"))
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
        Ok(ChainId::Custom(_)) | Ok(ChainId::Mainnet) | Err(_) => {
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
                                <p>Allowed networks: <code>SN_SEPOLIA</code>, <code>SN_DEVNET</code></p>
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
        Ok(ChainId::Sepolia) => ChainId::Sepolia,
        Ok(ChainId::Devnet) => ChainId::Devnet,
    };

    let payload_raw = query_params
        .get("payload")
        .unwrap_or(&String::new())
        .clone();

    // Parse payload as DepositPayload - return error if it fails
    let payload = match serde_json::from_str::<DepositPayload>(&payload_raw) {
        Ok(payload) => payload,
        Err(err) => {
            return Html(format!(
                r#"
                <!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>Invalid Payload</title>
                    <link rel="stylesheet" href="/static/styles.css">
                </head>
                <body>
                    <div class="container">
                        <header>
                            <h1>❌ Invalid Payload</h1>
                            <p>Payload must be deserializable as DepositPayload</p>
                        </header>
                        <main>
                            <div class="info-section">
                                <div class="info-item">
                                    <label>Error:</label>
                                    <span class="value">{}</span>
                                </div>
                                <div class="info-item payload-section">
                                    <label>Received Payload:</label>
                                    <pre class="payload-value">{}</pre>
                                </div>
                                <p><a href="/">Go back to home</a></p>
                            </div>
                        </main>
                    </div>
                </body>
                </html>
                "#,
                err, payload_raw
            ));
        }
    };

    let formatted_payload = serde_json::to_string_pretty(&payload).unwrap_or(payload_raw.clone());
    let payload_info = "<div class=\"info-item\"><label>Parsed Payload:</label><span class=\"value\">DepositPayload</span></div>".to_string();

    let on_chain_constants = ON_CHAIN_CONSTANTS
        .get(chain_id.as_str())
        .expect("a supported chain");

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
    let provider_url = match &chain_id {
        ChainId::Devnet => "http://localhost:5050".to_string(),
        ChainId::Sepolia => "https://starknet-sepolia.public.blastapi.io/rpc/v0_8".to_string(),
        ChainId::Custom(_) | ChainId::Mainnet => panic!("unsuported at the moment"),
    };

    let invoice_contract_abi: Vec<AbiEntry> = serde_json::from_str(
        r#"[{"type":"impl","name":"InvoicePaymentImpl","interface_name":"invoice_payment::IInvoicePayment"},{"type":"struct","name":"core::integer::u256","members":[{"name":"low","type":"core::integer::u128"},{"name":"high","type":"core::integer::u128"}]},{"type":"interface","name":"invoice_payment::IInvoicePayment","items":[{"type":"function","name":"pay_invoice","inputs":[{"name":"quote_id_hash","type":"core::felt252"},{"name":"expiry","type":"core::integer::u64"},{"name":"asset","type":"core::starknet::contract_address::ContractAddress"},{"name":"amount","type":"core::integer::u256"},{"name":"payee","type":"core::starknet::contract_address::ContractAddress"}],"outputs":[],"state_mutability":"external"}]},{"type":"event","name":"invoice_payment::InvoicePayment::Remittance","kind":"struct","members":[{"name":"asset","type":"core::starknet::contract_address::ContractAddress","kind":"key"},{"name":"payee","type":"core::starknet::contract_address::ContractAddress","kind":"key"},{"name":"invoice_id","type":"core::felt252","kind":"data"},{"name":"payer","type":"core::starknet::contract_address::ContractAddress","kind":"data"},{"name":"amount","type":"core::integer::u256","kind":"data"}]},{"type":"event","name":"invoice_payment::InvoicePayment::Event","kind":"enum","variants":[{"name":"Remittance","type":"invoice_payment::InvoicePayment::Remittance","kind":"nested"}]}]"#).unwrap();
    let ierc20_contract_abi: AbiEntry = serde_json::from_str(
        r#"{
    "name": "openzeppelin::token::erc20::interface::IERC20",
    "type": "interface",
    "items": [
      {
        "name": "name",
        "type": "function",
        "inputs": [],
        "outputs": [
          {
            "type": "core::felt252"
          }
        ],
        "state_mutability": "view"
      },
      {
        "name": "symbol",
        "type": "function",
        "inputs": [],
        "outputs": [
          {
            "type": "core::felt252"
          }
        ],
        "state_mutability": "view"
      },
      {
        "name": "decimals",
        "type": "function",
        "inputs": [],
        "outputs": [
          {
            "type": "core::integer::u8"
          }
        ],
        "state_mutability": "view"
      },
      {
        "name": "total_supply",
        "type": "function",
        "inputs": [],
        "outputs": [
          {
            "type": "core::integer::u256"
          }
        ],
        "state_mutability": "view"
      },
      {
        "name": "balance_of",
        "type": "function",
        "inputs": [
          {
            "name": "account",
            "type": "core::starknet::contract_address::ContractAddress"
          }
        ],
        "outputs": [
          {
            "type": "core::integer::u256"
          }
        ],
        "state_mutability": "view"
      },
      {
        "name": "allowance",
        "type": "function",
        "inputs": [
          {
            "name": "owner",
            "type": "core::starknet::contract_address::ContractAddress"
          },
          {
            "name": "spender",
            "type": "core::starknet::contract_address::ContractAddress"
          }
        ],
        "outputs": [
          {
            "type": "core::integer::u256"
          }
        ],
        "state_mutability": "view"
      },
      {
        "name": "transfer",
        "type": "function",
        "inputs": [
          {
            "name": "recipient",
            "type": "core::starknet::contract_address::ContractAddress"
          },
          {
            "name": "amount",
            "type": "core::integer::u256"
          }
        ],
        "outputs": [
          {
            "type": "core::bool"
          }
        ],
        "state_mutability": "external"
      },
      {
        "name": "transfer_from",
        "type": "function",
        "inputs": [
          {
            "name": "sender",
            "type": "core::starknet::contract_address::ContractAddress"
          },
          {
            "name": "recipient",
            "type": "core::starknet::contract_address::ContractAddress"
          },
          {
            "name": "amount",
            "type": "core::integer::u256"
          }
        ],
        "outputs": [
          {
            "type": "core::bool"
          }
        ],
        "state_mutability": "external"
      },
      {
        "name": "approve",
        "type": "function",
        "inputs": [
          {
            "name": "spender",
            "type": "core::starknet::contract_address::ContractAddress"
          },
          {
            "name": "amount",
            "type": "core::integer::u256"
          }
        ],
        "outputs": [
          {
            "type": "core::bool"
          }
        ],
        "state_mutability": "external"
      }
    ]
  }"#,
    )
    .unwrap();

    let deposit_data = DepositData {
        provider_url,
        invoice_contract: ConctractData {
            abi: invoice_contract_abi,
            address: on_chain_constants.invoice_payment_contract_address,
        },
        asset_contract: ConctractData {
            abi: vec![ierc20_contract_abi],
            address: payload.asset_contract_address,
        },
        quote_id_hash: payload.quote_id_hash,
        expiry: payload.expiry,
        amount_low: payload.amount.low,
        amount_high: payload.amount.high,
        payee: payload.payee,
    };

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
                        
                        <div class="info-item payload-section">
                            <label>Payload:</label>
                            <pre class="payload-value">{}</pre>
                        </div>

                        <div class="info-item">
                            <label>Wallet Status:</label>
                            <span id="wallet-status" class="value">Loading...</span>
                            <button id="wallet-connect-btn" class="wallet-btn" disabled>Connect Wallet</button>
                            <button id="deposit-btn" class="wallet-btn" hidden>Deposit</button>
                        </div>
                    </div>
                </main>

                <footer>
                    <p>Built with Rust + Axum + Starknet</p>
                </footer>
            </div>

            <script type="application/json" id="deposit-data">
                {}
            </script>
            <script src="/static/dist/main.bundle.js"></script>
        </body>
        </html>
        "#,
        params.method,
        params.network,
        chain_id,
        payload_info,
        formatted_payload,
        serde_json::to_string_pretty(&deposit_data).unwrap_or_else(|_| "{}".to_string()),
    );

    Html(html)
}
