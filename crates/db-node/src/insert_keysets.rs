use nuts::nut02::KeysetId;
use sqlx::{PgConnection, Postgres, QueryBuilder};

pub struct InsertKeysetsQueryBuilder<'args> {
    builder: QueryBuilder<'args, Postgres>,
    first: bool,
}

impl Default for InsertKeysetsQueryBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl InsertKeysetsQueryBuilder<'_> {
    pub fn new() -> Self {
        Self {
            builder: QueryBuilder::new(
                r#"INSERT INTO keyset (id, unit, active, max_order, derivation_path_index) VALUES "#,
            ),
            first: true,
        }
    }

    pub fn add_row<U: ToString>(&mut self, id: KeysetId, unit: U, max_order: u32, index: u32) {
        let id = id.as_i64();
        let unit = unit.to_string();
        let max_order = i32::from_be_bytes(max_order.to_be_bytes());
        let index = i32::from_be_bytes(index.to_be_bytes());

        if self.first {
            self.first = false;
        } else {
            self.builder.push(", ");
        }

        self.builder
            .push('(')
            .push_bind(id)
            .push(", ")
            .push_bind(unit)
            .push(", TRUE, ")
            .push_bind(max_order)
            .push(", ")
            .push_bind(index)
            .push(')');
    }

    pub async fn execute(mut self, conn: &mut PgConnection) -> Result<(), sqlx::Error> {
        _ = self
            .builder
            .push(r#" ON CONFLICT DO NOTHING;"#)
            .build()
            .execute(conn)
            .await?;

        Ok(())
    }
}
