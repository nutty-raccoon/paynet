use nuts::nut02::KeysetId;
use sqlx::{PgConnection, Postgres, QueryBuilder};

pub struct DeactivateKeysetsQueryBuilder<'args> {
    builder: QueryBuilder<'args, Postgres>,
    first: bool,
}

impl Default for DeactivateKeysetsQueryBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl DeactivateKeysetsQueryBuilder<'_> {
    pub fn new() -> Self {
        Self {
            builder: QueryBuilder::new("UPDATE keyset SET active = false WHERE id IN ("),
            first: true,
        }
    }

    pub fn add_keyset(&mut self, keyset_id: KeysetId) {
        if self.first {
            self.first = false;
        } else {
            self.builder.push(", ");
        }

        self.builder.push_bind(keyset_id.as_i64());
    }

    pub async fn execute(mut self, conn: &mut PgConnection) -> Result<(), sqlx::Error> {
        if self.first {
            return Ok(());
        }

        self.builder.push(");");
        self.builder.build().execute(conn).await?;
        Ok(())
    }
}
