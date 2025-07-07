use node_client::NodeClient;
use nuts::nut05::MeltQuoteState;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tonic::transport::Channel;

use crate::{db, errors::Error};

pub async fn melt_quote(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    method: String,
    quote_id: String,
) -> Result<Option<(MeltQuoteState, Vec<String>)>, Error> {
    let response = node_client
        .melt_quote_state(node_client::MeltQuoteStateRequest {
            method,
            quote: quote_id.clone(),
        })
        .await;

    match response {
        Err(status) if status.code() == tonic::Code::DeadlineExceeded => {
            let db_conn = pool.get()?;
            db::melt_quote::delete(&db_conn, &quote_id)?;
            Ok(None)
        }
        Ok(response) => {
            let response = response.into_inner();
            let state =
                MeltQuoteState::try_from(node_client::MeltQuoteState::try_from(response.state)?)?;

            let mut db_conn = pool.get()?;
            let tx = db_conn.transaction()?;
            match state {
                MeltQuoteState::Unpaid => {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    if now >= response.expiry {
                        db::melt_quote::delete(&tx, &quote_id)?;
                        return Ok(None);
                    }
                }
                MeltQuoteState::Pending => {}
                MeltQuoteState::Paid => {
                    if !response.transfer_ids.is_empty() {
                        let transfer_ids_to_store = serde_json::to_string(&response.transfer_ids)?;
                        db::melt_quote::register_transfer_ids(
                            &tx,
                            &quote_id,
                            &transfer_ids_to_store,
                        )?;
                    }
                }
            }

            db::melt_quote::update_state(&tx, &quote_id, response.state)?;
            tx.commit()?;

            Ok(Some((state, response.transfer_ids)))
        }
        Err(e) => Err(e)?,
    }
}
