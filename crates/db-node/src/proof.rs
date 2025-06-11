use nuts::{nut00::secret::Secret, nut01::PublicKey, nut02::KeysetId, nut07::ProofState};
use sqlx::PgConnection;

/// Return true if one of the provided secret
/// is already in db with state = SPENT
pub async fn is_any_already_spent(
    conn: &mut PgConnection,
    secret_derived_pubkeys: impl Iterator<Item = PublicKey>,
) -> Result<bool, sqlx::Error> {
    let ys: Vec<_> = secret_derived_pubkeys
        .map(|pk| pk.to_bytes().to_vec())
        .collect();

    let record = sqlx::query!(
        r#"SELECT EXISTS (
            SELECT * FROM proof WHERE y = ANY($1) AND state = $2
        ) AS "exists!";"#,
        &ys,
        ProofState::Spent as i16
    )
    .fetch_one(conn)
    .await?;

    Ok(record.exists)
}

pub async fn insert_proof(
    conn: &mut PgConnection,
    y: PublicKey,
    keyset_id: KeysetId,
    amount: i64,
    secret: Secret,
    unblind_signature: PublicKey,
    state: ProofState,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO proof (y, amount, keyset_id, secret, c, state)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        &y.to_bytes(),
        amount,
        keyset_id.as_i64(),
        secret.to_string(),
        &unblind_signature.to_bytes(),
        state as i16
    )
    .execute(conn)
    .await?;

    Ok(())
}
