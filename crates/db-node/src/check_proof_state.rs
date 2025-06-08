use nuts::{
    nut01::PublicKey,
    nut07::{ProofCheckState, ProofState},
};
use sqlx::PgConnection;

pub async fn get_proofs_by_y(
    conn: &mut PgConnection,
    ys: impl Iterator<Item = String>,
) -> Result<Vec<ProofCheckState>, sqlx::Error> {
    let ys: Vec<_> = ys.map(|pk| pk.into_bytes()).collect();

    let records = sqlx::query!(r#"SELECT y, state FROM proof WHERE y = ANY($1);"#, &ys)
        .fetch_all(conn)
        .await?;

    let proofs = records
        .into_iter()
        .map(|record| ProofCheckState {
            y: String::from_utf8(record.y).unwrap_or_default(),
            state: ProofState::from(record.state as i32),
            witness: None, // None for now. TODO: handle witness after other nuts are implemented
        })
        .collect::<Vec<_>>();

    Ok(proofs)
}
