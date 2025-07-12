use nuts::nut00::BlindSignature;
use nuts::nut01::PublicKey;
use sqlx::{PgConnection, Postgres, QueryBuilder};

pub struct InsertBlindSignaturesQueryBuilder<'args> {
    builder: QueryBuilder<'args, Postgres>,
    first: bool,
}

impl InsertBlindSignaturesQueryBuilder<'_> {
    pub fn new() -> Self {
        Self {
            builder: QueryBuilder::new(
                r#"INSERT INTO blind_signature (y, amount, keyset_id, c) VALUES "#,
            ),
            first: true,
        }
    }

    pub fn add_row(&mut self, blind_message: PublicKey, blind_signature: &BlindSignature) {
        let y = blind_message.to_bytes();
        let amount = blind_signature.amount.into_i64_repr();
        let keyset_id = blind_signature.keyset_id.as_i64();
        let c = blind_signature.c.to_bytes();

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
            .push_bind(c)
            .push(')');
    }

    pub async fn execute(mut self, conn: &mut PgConnection) -> Result<(), sqlx::Error> {
        _ = self.builder.push(r#";"#).build().execute(conn).await?;

        Ok(())
    }
}

impl Default for InsertBlindSignaturesQueryBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod query_builder {
    use num_traits::One;
    use nuts::{Amount, nut00::BlindSignature, nut01::PublicKey, nut02::KeysetId};

    use crate::InsertBlindSignaturesQueryBuilder;

    #[test]
    fn produce_expected_sql() {
        let mut builder = InsertBlindSignaturesQueryBuilder::new();
        let proof = BlindSignature {
            amount: Amount::one(),
            keyset_id: KeysetId::try_from(0x1i64).unwrap(),
            c: PublicKey::from_hex(
                "02194603ffa36356f4a56b7df9371fc3192472351453ec7398b8da8117e7c3e104",
            )
            .unwrap(),
        };

        let y = PublicKey::from_hex(
            "02194603ffa36356f4a56b7df9371fc3192472351453ec7398b8da8117e7c3e104",
        )
        .unwrap();

        builder.add_row(y, &proof);
        builder.add_row(y, &proof);
        let query = builder.builder.sql();
        assert_eq!(
            query,
            "INSERT INTO blind_signature (y, amount, keyset_id, c) VALUES ($1, $2, $3, $4), ($5, $6, $7, $8)"
        );
    }
}
