use crate::types::NodeUrl;
use nuts::{Amount, nut01::PublicKey};
use rusqlite::{
    Connection, Result, ToSql, params,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
};
use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

pub const CREATE_TABLE_WAD: &str = r#"
        CREATE TABLE IF NOT EXISTS wad (
            id BLOB NOT NULL,
            type TEXT NOT NULL CHECK (type IN ('IN', 'OUT')),
            status TEXT NOT NULL CHECK (status IN ('PENDING', 'CANCELLED', 'FINISHED', 'FAILED', 'PARTIAL')),
            node_url TEXT NOT NULL, 
            memo TEXT,
            created_at INTEGER NOT NULL,
            modified_at INTEGER NOT NULL,
            PRIMARY KEY (id, type)
        );

        CREATE INDEX wad_type ON wad(type);
        CREATE INDEX wad_status ON wad(status);
        CREATE INDEX wad_created_at ON wad(created_at);
    "#;

pub const CREATE_TABLE_WAD_PROOF: &str = r#"
        CREATE TABLE IF NOT EXISTS wad_proof (
            wad_id BLOB NOT NULL,
            proof_y BLOB(33) NOT NULL REFERENCES proof(y) ON DELETE CASCADE,
            PRIMARY KEY (wad_id, proof_y)
        );
    "#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WadType {
    IN,
    OUT,
}

impl ToSql for WadType {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            WadType::IN => Ok(ToSqlOutput::from("IN")),
            WadType::OUT => Ok(ToSqlOutput::from("OUT")),
        }
    }
}

impl FromSql for WadType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_str()? {
            "IN" => Ok(WadType::IN),
            "OUT" => Ok(WadType::OUT),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl std::fmt::Display for WadType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WadType::IN => write!(f, "IN"),
            WadType::OUT => write!(f, "OUT"),
        }
    }
}

/// State of a wad
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WadStatus {
    /// Wad has been seen but not processed yet
    Pending,
    /// Wad has been fully processed
    Finished,
    /// Wad processing failed
    Failed,
}

impl ToSql for WadStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            WadStatus::Pending => Ok(ToSqlOutput::from("PENDING")),
            WadStatus::Finished => Ok(ToSqlOutput::from("FINISHED")),
            WadStatus::Failed => Ok(ToSqlOutput::from("FAILED")),
        }
    }
}

impl FromSql for WadStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_str()? {
            "PENDING" => Ok(WadStatus::Pending),
            "FINISHED" => Ok(WadStatus::Finished),
            "FAILED" => Ok(WadStatus::Failed),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl std::fmt::Display for WadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WadStatus::Pending => write!(f, "PENDING"),
            WadStatus::Finished => write!(f, "FINISHED"),
            WadStatus::Failed => write!(f, "FAILED"),
        }
    }
}

#[derive(Debug, Clone)]

pub struct WadRecord {
    pub id: Uuid,
    pub r#type: WadType,
    pub status: WadStatus,
    pub node_url: String,
    pub memo: Option<String>,
    pub created_at: u64,
    pub modified_at: u64,
}

fn compute_wad_uuid(node_url: &NodeUrl, proofs_ys: &[PublicKey]) -> Uuid {
    const NAMESPACE_WAD: Uuid = Uuid::from_u128(336702331980467871995349228715494130514);

    // Doing this guarantee that the uuid is deterministic regardless of the oreder in which the ys are provided.
    // This is usefull as order is not deterministic when read form db.
    let mut sorted_proofs = proofs_ys.to_vec();
    sorted_proofs.sort();

    let mut buffer = Vec::new();
    buffer.extend_from_slice(node_url.0.as_str().as_bytes());
    for y in sorted_proofs {
        buffer.extend_from_slice(&y.to_bytes());
    }

    Uuid::new_v5(&NAMESPACE_WAD, &buffer)
}

pub fn register_wad(
    conn: &Connection,
    wad_type: WadType,
    node_url: &NodeUrl,
    memo: &Option<String>,
    proof_ys: &[PublicKey],
) -> Result<Uuid> {
    let wad_id = compute_wad_uuid(node_url, proof_ys);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    const INSERT_WAD: &str = r#"
        INSERT INTO wad 
            (id, type, status, node_url, memo, created_at, modified_at)
        VALUES 
            (?1, ?2, ?3, ?4, ?5, ?6, ?7)
    "#;
    let mut stmt = conn.prepare(INSERT_WAD)?;
    stmt.execute(params![
        wad_id,
        wad_type,
        WadStatus::Pending,
        node_url,
        memo,
        now,
        now,
    ])?;

    // Insert WAD-proof relationships
    const INSERT_WAD_PROOF: &str = r#"
        INSERT INTO wad_proof (wad_id, proof_y)
        VALUES (?1, ?2)
        ON CONFLICT DO NOTHING;
    "#;
    let mut stmt = conn.prepare(INSERT_WAD_PROOF)?;
    for proof_y in proof_ys {
        stmt.execute(params![wad_id, proof_y])?;
    }

    Ok(wad_id)
}

fn parse_wad_record(row: &rusqlite::Row) -> rusqlite::Result<WadRecord> {
    Ok(WadRecord {
        id: row.get(0)?,
        r#type: row.get(1)?,
        status: row.get(2)?,
        node_url: row.get(3)?,
        memo: row.get(4)?,
        created_at: row.get(5)?,
        modified_at: row.get(6)?,
    })
}

pub fn get_recent_wads(conn: &Connection, limit: u32) -> Result<Vec<WadRecord>> {
    const GET_RECENT_WADS: &str = r#"
        SELECT id, type, status, node_url, memo, created_at, modified_at
        FROM wad 
        ORDER BY created_at DESC 
        LIMIT ?1
    "#;
    let mut stmt = conn.prepare(GET_RECENT_WADS)?;
    let rows = stmt.query_map([limit], parse_wad_record)?;

    rows.collect::<Result<Vec<_>, _>>()
}

pub fn update_wad_status(conn: &Connection, wad_id: Uuid, status: WadStatus) -> Result<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    const UPDATE_WAD_STATUS: &str = r#"
        UPDATE wad 
        SET status = ?2, modified_at = ?3 
        WHERE id = ?1
    "#;
    let mut stmt = conn.prepare(UPDATE_WAD_STATUS)?;
    stmt.execute(params![wad_id, status, now])?;

    Ok(())
}

#[derive(Debug)]
pub(crate) struct SyncData {
    pub id: Uuid,
    pub r#type: WadType,
    pub node_url: NodeUrl,
}

pub(crate) fn get_pending_wads(conn: &Connection) -> Result<Vec<SyncData>> {
    const GET_PENDING_WADS: &str = r#"
        SELECT id, type, node_url 
        FROM wad 
        WHERE status = ?1
        ORDER BY created_at ASC
    "#;
    let mut stmt = conn.prepare(GET_PENDING_WADS)?;
    let rows = stmt.query_map([WadStatus::Pending], |r| {
        Ok(SyncData {
            id: r.get::<_, Uuid>(0)?,
            r#type: r.get::<_, WadType>(1)?,
            node_url: r.get::<_, NodeUrl>(2)?,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>()
}

pub fn get_proofs_ys_by_id(conn: &Connection, wad_id: Uuid) -> Result<Vec<PublicKey>> {
    const GET_WAD_PROOFS: &str = r#"
        SELECT proof_y FROM wad_proof WHERE wad_id = ?1
    "#;
    let mut stmt = conn.prepare(GET_WAD_PROOFS)?;
    let rows = stmt.query_map([wad_id], |row| {
        let y_bytes: Vec<u8> = row.get(0)?;
        PublicKey::from_slice(&y_bytes).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Blob, Box::new(e))
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>()
}

pub fn get_amounts_by_id<U: FromStr>(
    conn: &Connection,
    wad_id: Uuid,
) -> Result<Vec<(String, Amount)>> {
    const GET_WAD_UNIT_AMOUNTS: &str = r#"
        SELECT k.unit, SUM(p.amount) as total_amount
        FROM wad_proof wp
        JOIN proof p ON wp.proof_y = p.y
        JOIN keyset k ON p.keyset_id = k.id
        WHERE wp.wad_id = ?1
        GROUP BY k.unit
    "#;
    let mut stmt = conn.prepare(GET_WAD_UNIT_AMOUNTS)?;
    let rows = stmt.query_map([wad_id], |row| {
        let unit: String = row.get(0)?;
        let amount: Amount = row.get(1)?;
        Ok((unit, amount))
    })?;

    rows.collect::<Result<Vec<_>, _>>()
}
