use nuts::{
    Amount,
    nut00::{BlindSignature, BlindedMessage},
    nut01::PublicKey,
    nut02::KeysetId,
    traits::Unit,
};
use sqlx::{Connection, PgConnection, Pool, Postgres, Transaction};
use thiserror::Error;

mod insert_spent_proofs;
pub use insert_spent_proofs::InsertSpentProofsQueryBuilder;
mod insert_blind_signatures;
pub use insert_blind_signatures::InsertBlindSignaturesQueryBuilder;
mod insert_keysets;
pub use insert_keysets::InsertKeysetsQueryBuilder;
pub mod keyset;
pub mod melt_quote;
pub mod mint_quote;
pub mod payment_event;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to acquire lock")]
    Lock,
    #[error("Failed to compute y by running hash_on_curve")]
    HashOnCurve,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("Failed to convert the unit db record to the passed generic Unit type: \"{0}\"")]
    InvalidUnit(String),
    #[error("Failed to convert the db type into the runtime type")]
    DbToRuntimeConversion,
    #[error("Failed to convert the runtime type into the db type")]
    RuntimeToDbConversion,
}

/// Will return true if this secret has already been signed by us
pub async fn is_any_blind_message_already_used(
    conn: &mut PgConnection,
    blind_secrets: impl Iterator<Item = PublicKey>,
) -> Result<bool, sqlx::Error> {
    let ys: Vec<_> = blind_secrets.map(|pk| pk.to_bytes().to_vec()).collect();

    let record = sqlx::query!(
        r#"SELECT EXISTS (
            SELECT * FROM blind_signature WHERE y = ANY($1)
        ) AS "exists!";"#,
        &ys
    )
    .fetch_one(conn)
    .await?;

    Ok(record.exists)
}

/// Will return true if one of the provided secret
/// is already in db with state = SPENT
pub async fn is_any_proof_already_used(
    conn: &mut PgConnection,
    secret_derived_pubkeys: impl Iterator<Item = PublicKey>,
) -> Result<bool, sqlx::Error> {
    let ys: Vec<_> = secret_derived_pubkeys
        .map(|pk| pk.to_bytes().to_vec())
        .collect();

    let record = sqlx::query!(
        r#"SELECT EXISTS (
            SELECT * FROM proof WHERE y = ANY($1) AND state = 1
        ) AS "exists!";"#,
        &ys
    )
    .fetch_one(conn)
    .await?;

    Ok(record.exists)
}

/// Handle concurency at the database level
/// If one transaction alter a field that is used in another one
/// in a way that would result in a different statement output,
/// pgsql will either order them in a way that make it possible to execute,
/// or will make one fail.
/// See: https://www.postgresql.org/docs/current/transaction-iso.html#XACT-SERIALIZABLE
///
/// To be use at the very begining of a transaction.
///
/// If we were not doing this, we would have to acquire a lock for each proof, blind_signature
/// entry we read in db so that no other swap make use of them during this time.
/// I believe it's better to leave it to the db rather than manage it manualy.
async fn set_transaction_isolation_level_to_serializable(
    conn: &mut PgConnection,
) -> Result<(), sqlx::Error> {
    sqlx::query!("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;")
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn sum_amount_of_unit_in_circulation<U: Unit>(
    conn: &mut PgConnection,
    unit: U,
) -> Result<Amount, Error> {
    let record = sqlx::query!(
        r#"
            SELECT SUM(amount) AS "sum!: i64" FROM blind_signature 
            INNER JOIN keyset ON blind_signature.keyset_id = keyset.id
            WHERE keyset.unit = $1;
        "#,
        &unit.to_string()
    )
    .fetch_one(conn)
    .await?;

    let amount = Amount::from_i64_repr(record.sum);

    Ok(amount)
}

pub async fn begin_db_tx(
    pool: &Pool<Postgres>,
) -> Result<Transaction<'static, Postgres>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    set_transaction_isolation_level_to_serializable(&mut tx).await?;

    Ok(tx)
}

pub async fn start_db_tx_from_conn(
    conn: &mut PgConnection,
) -> Result<Transaction<'_, Postgres>, sqlx::Error> {
    let mut tx = conn.begin().await?;

    set_transaction_isolation_level_to_serializable(&mut tx).await?;

    Ok(tx)
}

pub async fn run_migrations(pool: &Pool<Postgres>) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./db/migrations/").run(pool).await
}

pub async fn get_blind_signature(
    conn: &mut PgConnection,
    blinded_message: &BlindedMessage,
) -> Result<BlindSignature, Error> {
    let amount = blinded_message.amount.into_i64_repr();
    let keyset_id = blinded_message.keyset_id.as_i64();
    let blinded_secret = blinded_message.blinded_secret.to_bytes();

    // Create a struct to represent the returned data with owned types
    #[derive(sqlx::FromRow)]
    struct BlindSignatureRow {
        amount: i64,
        keyset_id: Vec<u8>,
        c: Vec<u8>,
    }

    // Use the struct with query_as
    let row: BlindSignatureRow = sqlx::query_as(
        r#"
        SELECT amount, keyset_id, c
        FROM blind_signature
        WHERE amount = $1 AND keyset_id = $2 AND y = $3
        "#,
    )
    .bind(amount)
    .bind(keyset_id)
    .bind(blinded_secret)
    .fetch_one(conn)
    .await?;

    Ok(BlindSignature {
        amount: Amount::from_i64_repr(row.amount),
        keyset_id: KeysetId::from_bytes(&row.keyset_id)
            .map_err(|_| Error::DbToRuntimeConversion)?,
        c: PublicKey::from_slice(&row.c).map_err(|_| Error::DbToRuntimeConversion)?,
    })
}
