use node::MintQuoteResponse;
use rusqlite::{Connection, OptionalExtension, Result, params};

pub mod proof;

pub fn create_tables(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;

    const CREATE_TABLE_NODE: &str = r#"
        CREATE TABLE IF NOT EXISTS node (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL UNIQUE
        );

        CREATE INDEX node_url ON node(url); 
    "#;
    const CREATE_TABLE_KEYSET: &str = r#"
        CREATE TABLE IF NOT EXISTS keyset (
            id BLOB(8) PRIMARY KEY,
            node_id TEXT NOT NULL REFERENCES node(id) ON DELETE CASCADE,
            unit INTEGER NOT NULL,
            active BOOL NOT NULL
        );

        CREATE INDEX keyset_node_id ON keyset(node_id); 
        CREATE INDEX keyset_unit ON keyset(unit); 
        CREATE INDEX keyset_active ON keyset(active); 
    "#;
    const CREATE_TABLE_KEY: &str = r#"
        CREATE TABLE IF NOT EXISTS key (
            keyset_id BLOB(8) NOT NULL REFERENCES keyset(id) ON DELETE CASCADE,
            amount INTEGER NOT NULL,
            pubkey BLOB(33) NOT NULL,
            PRIMARY KEY (keyset_id, amount)
        );
    "#;
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
    tx.execute(CREATE_TABLE_KEYSET, ())?;
    tx.execute(CREATE_TABLE_KEY, ())?;
    tx.execute(CREATE_TABLE_MINT_QUOTE, ())?;
    tx.execute(proof::CREATE_TABLE_PROOF, ())?;

    tx.commit()?;

    Ok(())
}

pub fn store_mint_quote(
    conn: &Connection,
    method: String,
    amount: u64,
    unit: String,
    response: &MintQuoteResponse,
) -> Result<()> {
    const INSERT_NEW_MINT_QUOTE: &str = r#"
        INSERT INTO mint_quote
            (id, method, amount, unit, request, state, expiry)
        VALUES
            (?1, ?2, ?3, ?4, ?5, ?6, ?7);
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
pub fn set_mint_quote_state(conn: &Connection, quote_id: String, state: i32) -> Result<()> {
    const SET_MINT_QUOTE_STATE: &str = r#"
        UPDATE mint_quote
        SET state = ?2
        WHERE id = ?1;
    "#;

    conn.execute(SET_MINT_QUOTE_STATE, (&quote_id, state))?;

    Ok(())
}

pub fn insert_node(conn: &Connection, node_url: &str) -> Result<u32> {
    conn.execute(
        "INSERT INTO node (url) VALUES (?1) ON CONFLICT DO NOTHING;",
        [node_url],
    )?;

    let mut stmt = conn.prepare("SELECT id FROM node WHERE url = ?1;")?;
    let id = stmt.query_row([node_url], |r| r.get::<_, u32>(0))?;

    Ok(id)
}

pub fn upsert_node_keysets(
    conn: &mut Connection,
    node_id: u32,
    keysets: Vec<node::Keyset>,
) -> anyhow::Result<Vec<[u8; 8]>> {
    let tx = conn.transaction()?;

    tx.execute(
        r#"
        CREATE TEMPORARY TABLE IF NOT EXISTS _tmp_inserted (id INTEGER PRIMARY KEY);
        INSERT INTO _tmp_inserted (id) SELECT id FROM keyset;"#,
        (),
    )?;

    const UPSERT_NODE_KEYSET: &str = r#"
            INSERT INTO keyset (id, node_id, unit, active)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(id) DO UPDATE
                SET active=excluded.active
                WHERE active != excluded.active;
    "#;

    for keyset in keysets {
        let id: [u8; 8] = keyset
            .id
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid keyset id"))?;
        tx.execute(
            UPSERT_NODE_KEYSET,
            (id, node_id, keyset.unit, keyset.active),
        )?;
    }

    const GET_NEW_KEYSETS: &str = r#"
        SELECT id FROM keyset WHERE id NOT IN(SELECT id FROM _tmp_inserted);
    "#;

    let new_keyset_ids = {
        let mut stmt = tx.prepare(GET_NEW_KEYSETS)?;
        stmt.query_map((), |r| r.get::<_, [u8; 8]>(0))?
            .collect::<Result<Vec<_>>>()?
    };

    tx.execute("DELETE FROM _tmp_inserted", [])?;
    tx.commit()?;

    Ok(new_keyset_ids)
}

pub fn fetch_one_active_keyset_id_for_node_and_unit(
    conn: &Connection,
    node_id: u32,
    unit: &str,
) -> Result<Option<[u8; 8]>> {
    const FETCH_ONE_ACTIVE_KEYSET_FOR_NODE_AND_UNIT: &str = r#"
        SELECT id FROM keyset WHERE node_id = ? AND active = TRUE AND unit = ? LIMIT 1;
    "#;

    let mut stmt = conn.prepare(FETCH_ONE_ACTIVE_KEYSET_FOR_NODE_AND_UNIT)?;
    let mut rows_iter = stmt.query_map(params![node_id, unit], |row| row.get::<_, [u8; 8]>(0))?;

    rows_iter.next().transpose()
}

pub fn insert_keyset_keys<'a>(
    conn: &Connection,
    keyset_id: [u8; 8],
    keys: impl Iterator<Item = (u64, &'a str)>,
) -> Result<()> {
    const INSET_NEW_KEY: &str = r#"
        INSERT INTO key (keyset_id, amount, pubkey) VALUES (?1, ?2, ?3) ON CONFLICT DO NOTHING;
    "#;

    let mut stmt = conn.prepare(INSET_NEW_KEY)?;
    for (amount, pk) in keys {
        stmt.execute(params![keyset_id, amount, pk])?;
    }

    Ok(())
}

pub fn get_node_url(conn: &Connection, node_id: u32) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT url FROM node WHERE id = ?1 LIMIT 1")?;
    let opt_url = stmt
        .query_row([node_id], |r| r.get::<_, String>(0))
        .optional()?;

    Ok(opt_url)
}
