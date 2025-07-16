use crate::types::compact_wad::CompactWad;
use nuts::nut01::PublicKey;
use nuts::traits::Unit;
use rusqlite::{Connection, Result, params, ToSql, types::{ValueRef, ToSqlOutput, FromSqlResult, FromSqlError, FromSql}};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum WadStatus {
    Pending,
    Cancelled,
    Finished,
    Failed,
}

impl ToSql for WadStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            WadStatus::Pending => Ok(ToSqlOutput::from("PENDING")),
            WadStatus::Cancelled => Ok(ToSqlOutput::from("CANCELLED")),
            WadStatus::Finished => Ok(ToSqlOutput::from("FINISHED")),
            WadStatus::Failed => Ok(ToSqlOutput::from("FAILED")),
        }
    }
}

impl FromSql for WadStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_str()? {
            "PENDING" => Ok(WadStatus::Pending),
            "CANCELLED" => Ok(WadStatus::Cancelled),
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
            WadStatus::Cancelled => write!(f, "CANCELLED"),
            WadStatus::Finished => write!(f, "FINISHED"),
            WadStatus::Failed => write!(f, "FAILED"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WadRecord {
    pub id: i64,
    pub wad_type: WadType,
    pub status: WadStatus,
    pub wad_data: String,          // JSON string of CompactWad
    pub total_amount_json: String, // JSON array of unit/amount pairs
    pub memo: Option<String>,
    pub created_at: u64,
    pub modified_at: u64,
}

pub fn register_wad<U: Unit + serde::Serialize>(
    conn: &Connection,
    wad_type: WadType,
    wad: &CompactWad<U>,
    proof_ys: &[PublicKey],
) -> Result<i64> {
    let wad_data = serde_json::to_string(wad)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    let total_amount = wad
        .value()
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    let total_amount_json = serde_json::to_string(&[(wad.unit.to_string(), total_amount)])
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    const INSERT_WAD: &str = r#"
        INSERT INTO wad 
            (type, status, wad_data, total_amount_json, memo, created_at, modified_at)
        VALUES 
            (?1, ?2, ?3, ?4, ?5, ?6, ?7)
    "#;

    let mut stmt = conn.prepare(INSERT_WAD)?;
    stmt.execute(params![
        wad_type,
        WadStatus::Pending,
        wad_data,
        total_amount_json,
        wad.memo,
        now,
        now,
    ])?;

    let wad_id = conn.last_insert_rowid();

    // Insert WAD-proof relationships
    const INSERT_WAD_PROOF: &str = r#"
        INSERT INTO wad_proof (wad_id, proof_y)
        VALUES (?1, ?2)
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
        wad_type: row.get(1)?,
        status: row.get(2)?,
        wad_data: row.get(3)?,
        total_amount_json: row.get(4)?,
        memo: row.get(5)?,
        created_at: row.get(6)?,
        modified_at: row.get(7)?,
    })
}

pub fn get_recent_wads(conn: &Connection, limit: u32) -> Result<Vec<WadRecord>> {
    const GET_RECENT_WADS: &str = r#"
        SELECT id, type, status, wad_data, total_amount_json, memo, created_at, modified_at
        FROM wad 
        ORDER BY created_at DESC 
        LIMIT ?1
    "#;

    let mut stmt = conn.prepare(GET_RECENT_WADS)?;
    let rows = stmt.query_map([limit], parse_wad_record)?;

    rows.collect::<Result<Vec<_>, _>>()
}

pub fn update_wad_status(conn: &Connection, wad_id: i64, status: WadStatus) -> Result<()> {
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

pub fn get_pending_wads(conn: &Connection) -> Result<Vec<WadRecord>> {
    const GET_PENDING_WADS: &str = r#"
        SELECT id, type, status, wad_data, total_amount_json, memo, created_at, modified_at
        FROM wad 
        WHERE status = ?1
        ORDER BY created_at ASC
    "#;

    let mut stmt = conn.prepare(GET_PENDING_WADS)?;
    let rows = stmt.query_map([WadStatus::Pending], parse_wad_record)?;

    rows.collect::<Result<Vec<_>, _>>()
}

pub fn get_wad_proofs(conn: &Connection, wad_id: i64) -> Result<Vec<PublicKey>> {
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
