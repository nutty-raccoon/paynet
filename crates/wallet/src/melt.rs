use node_client::{MeltQuoteRequest, MeltQuoteResponse, NodeClient};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tonic::transport::Channel;

use crate::{db, errors::Error};

pub async fn create_quote(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    node_id: u32,
    method: String,
    unit: &str,
    request: String,
) -> Result<MeltQuoteResponse, Error> {
    let response = node_client
        .melt_quote(MeltQuoteRequest {
            method: method.clone(),
            unit: unit.to_string(),
            request: request.clone(),
        })
        .await?
        .into_inner();

    let db_conn = pool.get()?;
    db::melt_quote::store(&db_conn, node_id, method, request, &response)?;

    Ok(response)
}
