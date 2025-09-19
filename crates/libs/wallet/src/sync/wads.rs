use cashu_client::{CashuClient, CheckStateRequest};
use nuts::nut07::ProofState;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use uuid::Uuid;

use crate::{
    db::{self, wad::SyncData},
    errors::Error,
};

pub async fn pending_wads(
    pool: Pool<SqliteConnectionManager>,
    root_ca_certificate: Option<tonic::transport::Certificate>,
) -> Result<Vec<WadSyncResult>, Error> {
    let pending_wads = {
        let db_conn = pool.get()?;
        db::wad::get_pending_wads(&db_conn)?
    };

    let mut results = Vec::with_capacity(pending_wads.len());
    for sync_data in pending_wads {
        let wad_id = sync_data.id;
        let result = sync_single_wad(pool.clone(), sync_data, root_ca_certificate.clone()).await;

        results.push(WadSyncResult {
            wad_id,
            result: result.map_err(|e| e.to_string()),
        });
    }

    Ok(results)
}

async fn sync_single_wad(
    pool: Pool<SqliteConnectionManager>,
    sync_info: SyncData,
    root_ca_certificate: Option<tonic::transport::Certificate>,
) -> Result<Option<db::wad::WadStatus>, Error> {
    let SyncData {
        id: wad_id,
        r#type: _wad_type,
        node_url,
    } = sync_info;

    let proof_ys = {
        let db_conn = pool.get()?;
        db::wad::get_proofs_ys_by_id(&db_conn, wad_id)?
    };

    if proof_ys.is_empty() {
        return Ok(None);
    }

    let mut node_client = crate::connect_to_node(&node_url, root_ca_certificate).await?;

    let check_request = CheckStateRequest {
        ys: proof_ys.iter().map(|y| y.to_bytes().to_vec()).collect(),
    };

    let response = node_client.check_state(check_request).await?;
    let states = response.proof_check_states;
    let all_spent = states.iter().all(|state| match state.state {
        ProofState::Spent => true,
        ProofState::Unspent | ProofState::Pending => false,
        ProofState::Unspecified => false,
    });

    if all_spent {
        let db_conn = pool.get()?;
        db::wad::update_wad_status(&db_conn, wad_id, db::wad::WadStatus::Finished)?;
        Ok(Some(db::wad::WadStatus::Finished))
    } else {
        Ok(None)
    }
}

#[derive(Debug, Clone)]
pub struct WadSyncResult {
    pub wad_id: Uuid,
    pub result: Result<Option<db::wad::WadStatus>, String>,
}
