use nuts::{
    nut01::PublicKey,
    nut07::{ProofCheckState, ProofState},
};
use sqlx::PgConnection;

pub async fn get_proofs_by_y(
    conn: &mut PgConnection,
    ys: impl Iterator<Item = Vec<u8>>,
) -> Result<Vec<ProofCheckState>, sqlx::Error> {
    let records = sqlx::query!(
        r#"SELECT y, state FROM proof WHERE y = ANY($1);"#,
        &ys.collect::<Vec<_>>()
    )
    .fetch_all(conn)
    .await?;

    let proofs = records
        .into_iter()
        .map(|record| ProofCheckState {
            y: PublicKey::from_slice(&record.y).expect("Invalid public key format"),
            state: ProofState::from(record.state as i32),
        })
        .collect::<Vec<_>>();

    Ok(proofs)
}
