use node::MintQuoteResponse;
use rusqlite::{Connection, Result};

pub fn create_tables(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;

    const CREATE_TABLE_NODE: &str = r#"
        CREATE TABLE IF NOT EXISTS node (
            url TEXT PRIMARY KEY
        );"#;
    const CREATE_TABLE_KEYSET: &str = r#"
        CREATE TABLE IF NOT EXISTS keyset (
            id INTEGER PRIMARY KEY,
            node_url TEXT NOT NULL REFERENCES node(url) ON DELETE CASCADE,
            unit INTEGER NOT NULL,
            active BOOL NOT NULL
        );"#;
    const CREATE_TABLE_MINT_QUOTE: &str = r#"
        CREATE TABLE IF NOT EXISTS mint_quote (
            id BLOB(16) PRIMARY KEY,
            method TEXT NOT NULL,
            amount INTEGER NOT NULL,
            unit TEXT NOT NULL,
            request TEXT NOT NULL,
            state INTEGER NOT NULL CHECK (state IN (1, 2, 3)),
            expiry INTEGER NOT NULL
        );"#;

    tx.execute(CREATE_TABLE_NODE, ())?;
    tx.execute(CREATE_TABLE_MINT_QUOTE, ())?;
    tx.execute(CREATE_TABLE_KEYSET, ())?;

    tx.commit()?;

    Ok(())
}

pub fn store_mint_quote(
    conn: &mut Connection,
    method: String,
    amount: u64,
    unit: String,
    response: &MintQuoteResponse,
) -> Result<()> {
    const INSERT_NEW_MINT_QUOTE: &str = r#"
        INSERT INTO mint_quote
            (id, method, amount, unit, request, state, expiry)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7);
    "#;

    conn.execute(
        INSERT_NEW_MINT_QUOTE,
        (
            &response.quote,
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
pub fn set_mint_quote_state(conn: &mut Connection, quote_id: String, state: i32) -> Result<()> {
    const SET_MINT_QUOTE_STATE: &str = r#"
        UPDATE mint_quote
        SET state = $2
        WHERE id = $1;
    "#;

    conn.execute(SET_MINT_QUOTE_STATE, (&quote_id, state))?;

    Ok(())
}

pub fn insert_node(conn: &mut Connection, node_url: &str) -> Result<()> {
    const INSERT_NEW_NODE: &str = r#"
        INSERT INTO node (url) VALUES ($1) ON CONFLICT DO NOTHING;
    "#;

    conn.execute(INSERT_NEW_NODE, (node_url,))?;

    Ok(())
}

pub fn upsert_node_keysets(
    conn: &mut Connection,
    node_url: &str,
    keysets: Vec<node::Keyset>,
) -> anyhow::Result<()> {
    const UPSERT_NODE_KEYSET: &str = r#"
        INSERT INTO keyset (id, node_url, unit, active)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT(id) DO UPDATE SET active=excluded.active;
    "#;

    for keyset in keysets {
        let x: [u8; 8] = keyset
            .id
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid keyset id"))?;
        let id = i64::from_be_bytes(x);

        conn.execute(
            UPSERT_NODE_KEYSET,
            (id, node_url, keyset.unit, keyset.active),
        )?;
    }

    Ok(())
}

pub fn fetch_one_active_keyset_id_for_node_and_unit(
    conn: &mut Connection,
    node_url: &str,
    unit: String,
) -> Result<Option<i64>> {
    const FETCH_ONE_ACTIVE_KEYSET_FOR_NODE_AND_UNIT: &str = r#"
        SELECT id FROM keyset WHERE node_url = ? AND active = TRUE AND unit = ? LIMIT 1;
    "#;

    let mut stmt = conn.prepare(FETCH_ONE_ACTIVE_KEYSET_FOR_NODE_AND_UNIT)?;
    let mut rows_iter = stmt.query_map([node_url, &unit], |row| row.get::<_, i64>(0))?;

    rows_iter.next().transpose()
}
