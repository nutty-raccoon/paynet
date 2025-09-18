use nuts::nut02::KeysetId;
use rusqlite::{Connection, OptionalExtension, Result, params};

pub const CREATE_TABLE_KEYSET: &str = r#"
        CREATE TABLE IF NOT EXISTS keyset (
            id BLOB(8) PRIMARY KEY,
            node_id INTEGER NOT NULL REFERENCES node(id) ON DELETE CASCADE,
            unit TEXT NOT NULL,
            active BOOL NOT NULL,
            counter INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX keyset_node_id ON keyset(node_id);
        CREATE INDEX keyset_unit ON keyset(unit);
        CREATE INDEX keyset_active ON keyset(active);
    "#;

pub fn upsert_many_for_node(
    conn: &Connection,
    node_id: u32,
    keysets: Vec<cashu_client::ClientKeyset>,
) -> Result<Vec<KeysetId>> {
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
        let id = KeysetId::from_bytes(&keyset.id).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                keyset.id.len(),
                rusqlite::types::Type::Blob,
                Box::new(e),
            )
        })?;
        conn.execute(
            UPSERT_NODE_KEYSET,
            params![id, node_id, keyset.unit, keyset.active],
        )?;
    }

    const GET_NEW_KEYSETS: &str = r#"
        SELECT id FROM keyset WHERE id NOT IN(SELECT id FROM _tmp_inserted) AND node_id = ?1;
    "#;

    let new_keyset_ids = {
        let mut stmt = conn.prepare(GET_NEW_KEYSETS)?;
        stmt.query_map([node_id], |row| row.get::<_, KeysetId>(0))?
            .collect::<Result<Vec<_>>>()?
    };

    conn.execute("DELETE FROM _tmp_inserted", [])?;

    Ok(new_keyset_ids)
}

pub fn fetch_one_active_id_for_node_and_unit(
    conn: &Connection,
    node_id: u32,
    unit: &str,
) -> Result<Option<(KeysetId, u32)>> {
    const FETCH_ONE_ACTIVE_KEYSET_FOR_NODE_AND_UNIT: &str = r#"
        SELECT id, counter FROM keyset WHERE node_id = ? AND active = TRUE AND unit = ? LIMIT 1;
    "#;

    let mut stmt = conn.prepare(FETCH_ONE_ACTIVE_KEYSET_FOR_NODE_AND_UNIT)?;
    let result = stmt
        .query_row(params![node_id, unit], |row| {
            Ok((row.get::<_, KeysetId>(0)?, row.get(1)?))
        })
        .optional()?;

    Ok(result)
}

pub fn get_unit_by_id(conn: &Connection, keyset_id: KeysetId) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT unit FROM keyset WHERE id = ?1 LIMIT 1")?;
    let opt_unit = stmt
        .query_row(params![keyset_id], |r| r.get::<_, String>(0))
        .optional()?;

    Ok(opt_unit)
}

pub fn get_counter(conn: &Connection, keyset_id: KeysetId) -> Result<u32> {
    let mut stmt = conn.prepare("SELECT counter FROM keyset WHERE id = ?1 LIMIT 1")?;

    let counter = stmt.query_row(params![keyset_id], |r| r.get::<_, u32>(0))?;

    Ok(counter)
}

pub fn set_counter(conn: &Connection, keyset_id: KeysetId, counter: u32) -> Result<()> {
    let mut stmt = conn.prepare("UPDATE keyset SET counter = ?2 WHERE id = ?1")?;

    stmt.execute(params![keyset_id, counter])?;

    Ok(())
}

pub fn get_all_ids_for_node(conn: &Connection, node_id: u32) -> Result<Vec<KeysetId>> {
    const GET_ALL_KEYSETS_FOR_NODE: &str = r#"
        SELECT id FROM keyset WHERE node_id = ?1;
    "#;

    let mut stmt = conn.prepare(GET_ALL_KEYSETS_FOR_NODE)?;
    let keyset_ids = stmt
        .query_map([node_id], |row| row.get::<_, KeysetId>(0))?
        .collect::<Result<Vec<_>>>()?;

    Ok(keyset_ids)
}
