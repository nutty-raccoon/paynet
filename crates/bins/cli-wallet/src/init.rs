use std::io;

use rusqlite::Connection;

#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error(transparent)]
    SeedPhrase(#[from] wallet::seed_phrase::Error),
    #[error(transparent)]
    Wallet(#[from] wallet::wallet::Error),
    #[error(transparent)]
    IO(#[from] io::Error),
}

pub fn init(db_conn: &Connection) -> Result<(), InitError> {
    let seed_phrase = wallet::seed_phrase::create_random()?;

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

    wallet::wallet::init(db_conn, &seed_phrase)?;

    Ok(())
}
