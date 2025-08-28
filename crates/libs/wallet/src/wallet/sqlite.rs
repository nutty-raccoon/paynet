use bip39::Mnemonic;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::OptionalExtension;

use crate::seed_phrase;

const SINGLETON_SEED_PHRASE_KEY: &str = "seed_phrase";

#[derive(Debug, Clone)]
pub struct SeedPhraseManager {
    pool: Pool<SqliteConnectionManager>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to get a db connection from the pool: {0}")]
    R2D2(#[from] r2d2::Error),
    #[error("db error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("failed to parse stored value: {0}")]
    SeedPhrase(#[from] crate::seed_phrase::Error),
}

impl SeedPhraseManager {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Result<Self, Error> {
        {
            let conn = pool.get()?;
            const CREATE_TABLE_SEED_PHRASE: &str = r#"CREATE TABLE IF NOT EXISTS singleton (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );"#;
            conn.execute(CREATE_TABLE_SEED_PHRASE, ())?;
        }

        Ok(Self { pool })
    }
}

impl super::SeedPhraseManager for SeedPhraseManager {
    type Error = Error;

    fn store_seed_phrase(&self, seed_phrase: &bip39::Mnemonic) -> Result<(), Self::Error> {
        const STORE_SEED_PHRASE: &str = "INSERT INTO singleton (key, value) VALUES ($1, $2);";

        let conn = self.pool.get()?;
        conn.execute(
            STORE_SEED_PHRASE,
            (SINGLETON_SEED_PHRASE_KEY, seed_phrase.to_string()),
        )?;

        Ok(())
    }

    fn get_seed_phrase(&self) -> Result<Option<Mnemonic>, Self::Error> {
        const GET_SEED_PHRASE: &str = "SELECT value FROM singleton WHERE key = $1 LIMIT 1;";

        let conn = self.pool.get()?;
        let opt_seed_phrase_string = conn
            .query_row(GET_SEED_PHRASE, (SINGLETON_SEED_PHRASE_KEY,), |r| {
                r.get::<_, String>(0)
            })
            .optional()?;

        let opt_seed_phrase = match opt_seed_phrase_string {
            Some(s) => Some(seed_phrase::create_from_str(&s)?),
            None => None,
        };

        Ok(opt_seed_phrase)
    }
}
