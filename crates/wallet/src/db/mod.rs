use node::CREATE_TABLE_NODE;
use rusqlite::{Connection, OptionalExtension, Result, params};

use crate::types::{Amount, MeltResponseState, MintQuoteState, NodeUrl};

pub mod balance;
pub mod node;
pub mod proof;
pub mod types;

pub fn create_tables(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;

    const CREATE_TABLE_KEYSET: &str = r#"
        CREATE TABLE IF NOT EXISTS keyset (
            id BLOB(8) PRIMARY KEY,
            node_id INTEGER NOT NULL REFERENCES node(id) ON DELETE CASCADE,
            unit TEXT NOT NULL,
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
    const CREATE_TABLE_MELT_RESPONSE: &str = r#"
        CREATE TABLE IF NOT EXISTS melt_response (
            id BLOB (16) PRIMARY KEY,
            amount INTEGER NOT NULL,
            fee INT2 NOT NULL,
            state INT2 NOT NULL,
            expiry INTEGER NOT NULL
        )
    "#;

    tx.execute(CREATE_TABLE_NODE, ())?;
    tx.execute(CREATE_TABLE_KEYSET, ())?;
    tx.execute(CREATE_TABLE_KEY, ())?;
    tx.execute(CREATE_TABLE_MINT_QUOTE, ())?;
    tx.execute(CREATE_TABLE_MELT_RESPONSE, ())?;
    tx.execute(proof::CREATE_TABLE_PROOF, ())?;

    tx.commit()?;

    Ok(())
}

pub fn store_mint_quote(
    conn: &Connection,
    method: String,
    amount: Amount,
    unit: String,
    response: &::node::MintQuoteResponse,
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
            MintQuoteState::Pending,
            response.expiry,
        ),
    )?;

    Ok(())
}

pub fn set_mint_quote_state(
    conn: &Connection,
    quote_id: String,
    state: MintQuoteState,
) -> Result<()> {
    const SET_MINT_QUOTE_STATE: &str = r#"
        UPDATE mint_quote
        SET state = ?2
        WHERE id = ?1;
    "#;

    conn.execute(SET_MINT_QUOTE_STATE, (&quote_id, state))?;

    Ok(())
}

pub fn upsert_node_keysets(
    conn: &Connection,
    node_id: u32,
    keysets: Vec<::node::Keyset>,
) -> Result<Vec<[u8; 8]>> {
    conn.execute(
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
        let id: [u8; 8] = keyset.id.try_into().map_err(|e: Vec<u8>| {
            rusqlite::Error::ToSqlConversionFailure(
                format!("Invalid keyset ID length: {}", e.len()).into(),
            )
        })?;
        conn.execute(
            UPSERT_NODE_KEYSET,
            (id, node_id, keyset.unit, keyset.active),
        )?;
    }

    const GET_NEW_KEYSETS: &str = r#"
        SELECT id FROM keyset WHERE id NOT IN(SELECT id FROM _tmp_inserted);
    "#;

    let new_keyset_ids = {
        let mut stmt = conn.prepare(GET_NEW_KEYSETS)?;
        stmt.query_map([], |row| row.get(0))?
            .map(|res| res)
            .collect::<Result<Vec<_>>>()?
    };

    conn.execute("DELETE FROM _tmp_inserted", [])?;

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
    keys: impl Iterator<Item = (Amount, &'a str)>,
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

pub fn get_node_url(conn: &Connection, node_id: u32) -> Result<Option<NodeUrl>> {
    let mut stmt = conn.prepare("SELECT url FROM node WHERE id = ?1 LIMIT 1")?;
    let opt_url = stmt
        .query_row([node_id], |r| r.get::<_, NodeUrl>(0))
        .optional()?;

    Ok(opt_url)
}

pub fn get_keyset_unit(conn: &Connection, keyset_id: [u8; 8]) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT unit FROM keyset WHERE id = ?1 LIMIT 1")?;
    let opt_unit = stmt
        .query_row([keyset_id], |r| r.get::<_, String>(0))
        .optional()?;

    Ok(opt_unit)
}

pub fn register_melt_quote(conn: &Connection, response: &::node::MeltResponse) -> Result<()> {
    const INSERT_MELT_RESPONSE: &str = r#"
            INSERT INTO melt_response (
                id, amount, fee, state, expiry
            ) VALUES (?1, ?2, ?3, ?4, ?5)
            "#;

    conn.execute(
        INSERT_MELT_RESPONSE,
        [
            &response.quote,
            &response.amount,
            &response.fee,
            &MeltResponseState::Pending,
            &response.expiry,
        ],
    )?;

    Ok(())
}
