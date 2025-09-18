use nuts::{Amount, nut04::MintQuoteState};
use rusqlite::{Connection, OptionalExtension, Result, params};

pub const CREATE_TABLE_MINT_QUOTE: &str = r#"
        CREATE TABLE IF NOT EXISTS mint_quote (
            id BLOB(16) PRIMARY KEY,
            node_id INTEGER NOT NULL REFERENCES node(id) ON DELETE CASCADE,
            method TEXT NOT NULL,
            amount INTEGER NOT NULL,
            unit TEXT NOT NULL,
            request TEXT NOT NULL,
            state INTEGER NOT NULL CHECK (state IN (1, 2, 3)),
            expiry INTEGER NOT NULL
        );"#;

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
    response: &nuts::nut04::MintQuoteResponse<String>,
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
pub fn set_state(conn: &Connection, quote_id: &str, state: MintQuoteState) -> Result<()> {
    const SET_MINT_QUOTE_STATE: &str = r#"
        UPDATE mint_quote
        SET state = ?2
        WHERE id = ?1;
    "#;

    conn.execute(SET_MINT_QUOTE_STATE, (quote_id, state))?;

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

pub fn get(conn: &Connection, node_id: u32, quote_id: &str) -> Result<Option<MintQuote>> {
    const GET_MINT_QUOTE: &str = r#"
        SELECT * FROM mint_quote
        WHERE node_id = ?1 AND id = ?2;
    "#;

    let quote = conn
        .query_row(GET_MINT_QUOTE, params![node_id, quote_id], |r| {
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
        })
        .optional()?;

    Ok(quote)
}

#[derive(Debug, Clone)]
pub struct PendingMintQuote {
    pub id: String,
    pub method: String,
    pub amount: Amount,
    pub unit: String,
    pub request: String,
    pub state: MintQuoteState,
    pub expiry: u64,
}

pub fn get_pendings(conn: &Connection) -> Result<Vec<(u32, Vec<PendingMintQuote>)>> {
    const GET_PENDING_QUOTES: &str = r#"
        SELECT *
        FROM mint_quote
        WHERE state = ?1 OR state = ?2;
        ORDER BY node_id;
    "#;

    let mut stmt = conn.prepare(GET_PENDING_QUOTES)?;
    let mut rows = stmt.query([MintQuoteState::Unpaid, MintQuoteState::Paid])?;

    let mut quote_per_node: Vec<(u32, Vec<PendingMintQuote>)> = Vec::new();
    while let Some(r) = rows.next()? {
        let node_id = r.get::<_, u32>(1)?;
        let pending_mint_quote = PendingMintQuote {
            id: r.get::<_, _>(0)?,
            method: r.get::<_, _>(2)?,
            amount: r.get::<_, _>(3)?,
            unit: r.get::<_, _>(4)?,
            request: r.get::<_, _>(5)?,
            state: r.get::<_, _>(6)?,
            expiry: r.get::<_, _>(7)?,
        };

        match quote_per_node.iter().position(|v| v.0 == node_id) {
            Some(p) => quote_per_node[p].1.push(pending_mint_quote),
            None => quote_per_node.push((node_id, vec![pending_mint_quote])),
        }
    }

    Ok(quote_per_node)
}
