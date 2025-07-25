use std::io;

use rusqlite::Connection;

#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    SeedPhrase(#[from] wallet::seed_phrase::Error),
    #[error(transparent)]
    IO(#[from] io::Error),
}

pub fn init(db_conn: &Connection) -> Result<(), InitError> {
    let seed_phrase = wallet::seed_phrase::create_random()?;
    let private_key = wallet::seed_phrase::derive_private_key(&seed_phrase)?;

    let wallet = wallet::db::wallet::Wallet {
        seed_phrase: seed_phrase.to_string(),
        private_key: private_key.to_string(),
        is_restored: false,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        updated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    let mut input = String::new();
    println!(
        "Here is your seed phrase: {}\nWith it your will be able to recover your funds, should you lose access to this device or destroy your local database.\n Make sure to save it somewhere safe.",
        seed_phrase
    );
    println!("Have you stored this seed phrase in a safe place? (y/n)");

    loop {
        std::io::stdin().read_line(&mut input)?;
        let has_user_saved_seed_phrase = input.trim().to_lowercase();

        if has_user_saved_seed_phrase == "y" || has_user_saved_seed_phrase == "yes" {
            break;
        }

        println!(
            "Please save your seed phrase.\nEnter 'y' or 'yes' to finalize your wallet once it is done."
        );

        input.clear();
    }

    wallet::db::wallet::create(db_conn, wallet)?;

    println!("Wallet saved!");

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum RestoreError {
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    SeedPhrase(#[from] wallet::seed_phrase::Error),
}

pub fn restore(db_conn: &Connection, seed_phrase: String) -> Result<(), RestoreError> {
    let seed_phrase = wallet::seed_phrase::create_from_str(&seed_phrase)?;
    let private_key = wallet::seed_phrase::derive_private_key(&seed_phrase)?;

    let wallet = wallet::db::wallet::Wallet {
        seed_phrase: seed_phrase.to_string(),
        private_key: private_key.to_string(),
        is_restored: true,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        updated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    wallet::db::wallet::create(db_conn, wallet)?;

    println!("Wallet saved!");

    Ok(())
}
