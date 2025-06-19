use nuts::{Amount, nut04::MintQuoteState};
use rusqlite::{Connection, Result, params};

#[derive(Debug)]
pub struct MintQuote {
    pub id: String,
    pub node_id: u32,
    pub method: String,
    pub amount: Amount,
    pub unit: String,
    pub request: String,
    pub state: MintQuoteState,
    pub expiry: u64,
}

pub fn store(
    conn: &Connection,
    node_id: u32,
    method: String,
    amount: Amount,
    unit: &str,
    response: &node_client::MintQuoteResponse,
) -> Result<()> {
    const INSERT_NEW_MINT_QUOTE: &str = r#"
        INSERT INTO mint_quote
            (id, node_id, method, amount, unit, request, state, expiry)
        VALUES
            (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);
    "#;

    conn.execute(
        INSERT_NEW_MINT_QUOTE,
        (
            &response.quote,
            node_id,
            method,
            amount,
            unit,
            &response.request,
            response.state,
            response.expiry,
        ),
    )?;

    Ok(())
}
pub fn set_state(conn: &Connection, quote_id: String, state: i32) -> Result<()> {
    const SET_MINT_QUOTE_STATE: &str = r#"
        UPDATE mint_quote
        SET state = ?2
        WHERE id = ?1;
    "#;

    conn.execute(SET_MINT_QUOTE_STATE, (&quote_id, state))?;

    Ok(())
}

pub fn delete(conn: &Connection, quote_id: &str) -> Result<()> {
    const DELETE_MINT_QUOTE_STATE: &str = r#"
        DELETE FROM mint_quote
        WHERE id = ?1;
    "#;

    conn.execute(DELETE_MINT_QUOTE_STATE, [quote_id])?;

    Ok(())
}

pub fn get(conn: &Connection, node_id: u32, quote_id: &str) -> Result<MintQuote> {
    const GET_MINT_QUOTE_STATE: &str = r#"
        SELECT * FROM mint_quote
        WHERE node_id = ?1 AND id = ?2;
    "#;

    let quote = conn.query_row(GET_MINT_QUOTE_STATE, params![node_id, quote_id], |r| {
        Ok(MintQuote {
            id: r.get::<_, _>(0)?,
            node_id: r.get::<_, _>(1)?,
            method: r.get::<_, _>(2)?,
            amount: r.get::<_, _>(3)?,
            unit: r.get::<_, _>(4)?,
            request: r.get::<_, _>(5)?,
            state: r.get::<_, _>(6)?,
            expiry: r.get::<_, _>(7)?,
        })
    })?;

    Ok(quote)
}

#[allow(clippy::type_complexity)]
pub fn get_pendings(
    conn: &Connection,
) -> Result<Vec<(u32, Vec<(String, String, i32, String, u64)>)>> {
    const GET_PENDING_QUOTES: &str = r#"
        SELECT node_id, method, id, state, unit, amount FROM mint_quote WHERE state = 1 OR state = 2;
    "#;

    let mut stmt = conn.prepare(GET_PENDING_QUOTES)?;
    let mut rows = stmt.query([])?;

    let mut quote_per_node: Vec<(u32, Vec<(String, String, i32, String, u64)>)> = Vec::new();
    while let Some(row) = rows.next()? {
        let node_id = row.get::<_, u32>(0)?;
        let method = row.get::<_, String>(1)?;
        let id = row.get::<_, String>(2)?;
        let state = row.get::<_, i32>(3)?;
        let unit = row.get::<_, String>(4)?;
        let amount = row.get::<_, u64>(5)?;

        match quote_per_node.iter().position(|v| v.0 == node_id) {
            Some(p) => quote_per_node[p].1.push((method, id, state, unit, amount)),
            None => quote_per_node.push((node_id, vec![(method, id, state, unit, amount)])),
        }
    }

    Ok(quote_per_node)
}
