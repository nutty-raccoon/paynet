use std::collections::HashMap;

use nuts::{
    nut00::{Proof, secret::Secret},
    nut01::PublicKey,
    nut02::KeysetId,
    nut07::ProofState,
};

use sqlx::{PgConnection, Postgres, QueryBuilder, Row};
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

pub async fn get_proofs_by_ids(
    conn: &mut PgConnection,
    ys: &[Vec<u8>],
) -> Result<HashMap<Vec<u8>, ProofState>, sqlx::Error> {
    if ys.is_empty() {
        return Ok(HashMap::new());
    }

    let placeholders: String = (1..=ys.len())
        .map(|i| format!("${}", i))
        .collect::<Vec<_>>()
        .join(", ");

    let sql = format!(
        "SELECT y, state FROM proof WHERE y = ANY(ARRAY[{}])",
        placeholders
    );

    let mut query = sqlx::query(&sql);
    for y in ys {
        query = query.bind(y.as_slice());
    }

    let records = query.fetch_all(conn).await?;

    // Build HashMap from results
    let mut results: HashMap<Vec<u8>, ProofState> = HashMap::new();

    for record in records {
        let y: Vec<u8> = record.get("y");
        let state: i16 = record.get("state");
        results.insert(y, (state as i32).into());
    }

    Ok(results)
}

/// Generate a query following this model:
/// INSERT INTO proof (y, amount, keyset_id, secret, c, state)
/// VALUES  ($1, $2, $3, $4, $5, 1), ($6, $7, $8, $9, $10, 1)
///  ON CONFLICT (y) WHERE state = 0 DO UPDATE SET state = 1;
///
/// Meaning it will fail if a state is already set to 1 (SPENT).
/// Otherwise it will either inset new proofs AS SPENT,
/// or or update previously existing UNSPENT proofs to SPENT.
pub struct InsertSpentProofsQueryBuilder<'args> {
    builder: QueryBuilder<'args, Postgres>,
    first: bool,
}

impl<'args> InsertSpentProofsQueryBuilder<'args> {
    pub fn new() -> Self {
        Self {
            builder: QueryBuilder::new(
                r#"INSERT INTO proof (y, amount, keyset_id, secret, c, state) VALUES "#,
            ),
            first: true,
        }
    }

    pub fn add_row(&mut self, y: &PublicKey, proof: &'args Proof) {
        let y = y.to_bytes();
        let amount = proof.amount.into_i64_repr();
        let keyset_id = proof.keyset_id.as_i64();
        let secret: &str = proof.secret.as_ref();
        let c = proof.c.to_bytes();
        let state = ProofState::Spent as i16;

        if self.first {
            self.first = false;
        } else {
            self.builder.push(", ");
        }

        self.builder
            .push('(')
            .push_bind(y)
            .push(", ")
            .push_bind(amount)
            .push(", ")
            .push_bind(keyset_id)
            .push(", ")
            .push_bind(secret)
            .push(", ")
            .push_bind(c)
            .push(", ")
            .push(state)
            .push(')');
    }

    // this will insert the proofs as SPENT, or update existing UNSPENT proofs to SPENT
    // if they are already in the database.
    pub async fn execute(mut self, conn: &mut PgConnection) -> Result<(), sqlx::Error> {
        _ = self
            .builder
            .push(format!(
                "ON CONFLICT (y) WHERE state = {} DO UPDATE SET state = {};",
                ProofState::Unspent as i16,
                ProofState::Spent as i16
            ))
            .build()
            .execute(conn)
            .await?;

        Ok(())
    }
}

impl Default for InsertSpentProofsQueryBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod query_builder {
    use num_traits::One;
    use nuts::{
        Amount,
        nut00::{Proof, secret::Secret},
        nut01::PublicKey,
        nut02::KeysetId,
    };

    use crate::InsertSpentProofsQueryBuilder;

    #[test]
    fn produce_expected_sql() {
        let mut builder = InsertSpentProofsQueryBuilder::new();
        let proof = Proof {
            amount: Amount::one(),
            keyset_id: KeysetId::try_from(0x1i64).unwrap(),
            secret: Secret::default(),
            c: PublicKey::from_hex(
                "02194603ffa36356f4a56b7df9371fc3192472351453ec7398b8da8117e7c3e104",
            )
            .unwrap(),
        };
        let y = proof.y().unwrap();

        builder.add_row(&y, &proof);
        builder.add_row(&y, &proof);
        let query = builder.builder.sql();

        let spent_as_i16 = nuts::nut07::ProofState::Spent as i16;
        assert_eq!(
            query,
            format!(
                "INSERT INTO proof (y, amount, keyset_id, secret, c, state) VALUES ($1, $2, $3, $4, $5, {}), ($6, $7, $8, $9, $10, {})",
                spent_as_i16, spent_as_i16
            )
        );
    }
}
