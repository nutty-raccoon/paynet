use nuts::{nut00::Proof, nut01::PublicKey};
use sqlx::{PgConnection, Postgres, QueryBuilder};

/// Generate a query following this model:
/// INSERT INTO proof (y, amount, keyset_id, secret, c, state)
/// VALUES  ($1, $2, $3, $4, $5, 1), ($6, $7, $8, $9, $10, 1)
///  ON CONFLICT (y) WHERE state = 0 DO UPDATE SET state = 1;
///
/// Meaning it will fail if a state is already set to 1 (SPENT).
/// Otherwise it will either inset new proofs as SPENT,
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
            .push("1") // '1' is enum value for SPENT
            .push(')');
    }

    pub async fn execute(mut self, conn: &mut PgConnection) -> Result<(), sqlx::Error> {
        _ = self
            .builder
            // 0 is enum for UNSPENT
            // 1 is enum for SPENT
            .push(r#" ON CONFLICT (y) WHERE state = 0 DO UPDATE SET state = 1;"#) // TODO: make sure this is ok
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
        assert_eq!(
            query,
            "INSERT INTO proof (y, amount, keyset_id, secret, c, state) VALUES ($1, $2, $3, $4, $5, 1), ($6, $7, $8, $9, $10, 1)"
        );
    }
}
