use anyhow::Result;
use node::NodeClient;
use node::{MintQuoteRequest, MintQuoteResponse};
use rusqlite::Connection;

mod db;
pub use db::create_tables;

pub async fn create_mint_quote(
    db_conn: &mut Connection,
    node_client: &mut NodeClient<tonic::transport::Channel>,
    method: String,
    amount: u64,
    unit: String,
) -> Result<MintQuoteResponse> {
    let response = node_client
        .mint_quote(MintQuoteRequest {
            method: method.clone(),
            amount,
            unit: unit.clone(),
            description: None,
        })
        .await?
        .into_inner();

    db::store_mint_quote(db_conn, method, amount, unit, &response)?;

    Ok(response)
}
