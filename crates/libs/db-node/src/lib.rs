use nuts::nut01::PublicKey;
use sqlx::{Connection, PgConnection, Pool, Postgres, Transaction};
use thiserror::Error;

pub mod gauge;
mod insert_blind_signatures;
pub use insert_blind_signatures::InsertBlindSignaturesQueryBuilder;
mod insert_keysets;
pub use insert_keysets::InsertKeysetsQueryBuilder;
pub mod blind_signature;
pub mod keyset;
pub mod melt_payment_event;
pub mod melt_quote;
pub mod mint_payment_event;
pub mod mint_quote;
pub mod proof;
pub use proof::InsertSpentProofsQueryBuilder;

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

#[derive(Debug)]
pub struct PaymentEvent {
    pub block_id: String,
    pub tx_hash: String,
    pub index: i64,
    pub asset: String,
    pub payee: String,
    pub invoice_id: [u8; 32],
    pub payer: String,
    pub amount_low: String,
    pub amount_high: String,
}
