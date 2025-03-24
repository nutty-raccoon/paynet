use sqlx::{PgPool, postgres::PgPoolOptions};

use super::Error;

pub async fn connect_to_db_and_run_migrations(pg_url: &str) -> Result<PgPool, Error> {
    let pool = PgPoolOptions::new()
        .max_connections(32)
        .min_connections(6)
        .connect(pg_url)
        .await
        .map_err(Error::DbConnect)?;

    db_node::run_migrations(&pool)
        .await
        .map_err(Error::DbMigrate)?;

    Ok(pool)
}
