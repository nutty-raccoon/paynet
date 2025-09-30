use askama::Template;
use serde::{Deserialize, Serialize};
use starknet_core::types::{contract::AbiEntry, Felt};

#[derive(Serialize, Deserialize, Debug)]
pub struct RouteParams {
    pub method: String,
    pub network: String,
}

#[derive(Template)]
#[template(path = "invalid_method.html")]
pub struct InvalidMethodTemplate {
    pub method: String,
}

#[derive(Template)]
#[template(path = "invalid_network.html")]
pub struct InvalidNetworkTemplate {
    pub network: String,
}

#[derive(Template)]
#[template(path = "invalid_payload.html")]
pub struct InvalidPayloadTemplate {
    pub error: String,
    pub payload_raw: String,
}

#[derive(Template)]
#[template(path = "deposit.html")]
pub struct DepositTemplate {
    pub method: String,
    pub network: String,
    pub formatted_payload: String,
    pub deposit_data: DepositData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConctractData {
    pub abi: Vec<AbiEntry>,
    pub address: Felt,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepositData {
    pub provider_url: String,
    pub invoice_contract: ConctractData,
    pub asset_contract: ConctractData,
    pub quote_id_hash: Felt,
    pub expiry: Felt,
    pub amount_low: Felt,
    pub amount_high: Felt,
    pub payee: Felt,
}

#[derive(Template)]
#[template(path = "salto_landing.html")]
pub struct SaltoLandingTemplate {}
