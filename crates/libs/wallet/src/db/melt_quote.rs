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

pub fn register_transfer_ids(conn: &Connection, quote_id: &str, transfer_ids: &str) -> Result<()> {
    const INSERT_TRANSFER_IDS: &str = r#"
       UPDATE melt_quote SET transfer_ids = ?2 WHERE id = ?1; 
    "#;

    conn.execute(INSERT_TRANSFER_IDS, [quote_id, transfer_ids])?;

    Ok(())
}

#[derive(Debug, Clone)]
pub struct PendingMeltQuote {
    pub id: String,
    pub state: MeltQuoteState,
    pub expiry: u64,
}

#[allow(clippy::type_complexity)]
pub fn get_pendings(conn: &Connection) -> Result<Vec<(u32, Vec<PendingMeltQuote>)>> {
    const GET_PENDING_MELT_QUOTES: &str = r#"
        SELECT node_id, id, state, expiry 
        FROM melt_quote 
        WHERE state = ? OR state = ?
        ORDER BY node_id;
    "#;

    let mut stmt = conn.prepare(GET_PENDING_MELT_QUOTES)?;
    let mut rows = stmt.query([1, 2])?; // MlqsUnpaid = 1, MlqsPending = 2

    let mut quote_per_node: Vec<(u32, Vec<PendingMeltQuote>)> = Vec::new();
    while let Some(row) = rows.next()? {
        let node_id = row.get::<_, u32>(0)?;
        let pending_melt_quote = PendingMeltQuote {
            id: row.get(1)?,
            state: row.get(2)?,
            expiry: row.get(3)?,
        };

        match quote_per_node.iter().position(|v| v.0 == node_id) {
            Some(p) => quote_per_node[p].1.push(pending_melt_quote),
            None => quote_per_node.push((node_id, vec![pending_melt_quote])),
        }
    }

    Ok(quote_per_node)
}

pub fn update_state(conn: &Connection, quote_id: &str, state: i32) -> Result<()> {
    const UPDATE_MELT_QUOTE_STATE: &str = r#"
        UPDATE melt_quote
        SET state = ?2
        WHERE id = ?1;
    "#;

    conn.execute(UPDATE_MELT_QUOTE_STATE, (quote_id, state))?;

    Ok(())
}

pub fn delete(conn: &Connection, quote_id: &str) -> Result<()> {
    const DELETE_MELT_QUOTE: &str = r#"
        DELETE FROM melt_quote
        WHERE id = ?1;
    "#;

    conn.execute(DELETE_MELT_QUOTE, [quote_id])?;

    Ok(())
}
