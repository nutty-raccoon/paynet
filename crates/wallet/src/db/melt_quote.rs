use nuts::nut05::MeltQuoteState;
use rusqlite::{Connection, Result};

#[derive(Debug)]
pub struct MeltQuote {
    pub id: String,
    pub node_id: u32,
    pub method: String,
    pub unit: String,
    pub request: String,
    pub state: MeltQuoteState,
    pub expiry: u64,
}

pub fn store(
    conn: &Connection,
    node_id: u32,
    method: String,
    request: String,
    response: &node_client::MeltQuoteResponse,
) -> Result<()> {
    const INSERT_NEW_MELT_QUOTE: &str = r#"
        INSERT INTO melt_quote
            (id, node_id, method, amount, unit, request, state, expiry)
        VALUES
            (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);
    "#;

    conn.execute(
        INSERT_NEW_MELT_QUOTE,
        (
            &response.quote,
            node_id,
            method,
            response.amount,
            &response.unit,
            &request,
            response.state,
            response.expiry,
        ),
    )?;

    Ok(())
}

pub fn register_transfer_ids(
    conn: &Connection,
    quote_id: String,
    transfer_ids: String,
) -> Result<()> {
    const INSERT_TRANSFER_IDS: &str = r#"
       INSERT INTO melt_quote (transfer_ids) VALUES (?2) WHERE id = ?1; 
    "#;

    conn.execute(INSERT_TRANSFER_IDS, [quote_id, transfer_ids])?;

    Ok(())
}
