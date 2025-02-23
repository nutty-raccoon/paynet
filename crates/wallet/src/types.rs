use nuts::{
    Amount,
    dhke::blind_message,
    nut00::{BlindedMessage, secret::Secret},
    nut01::SecretKey,
    nut02::KeysetId,
};

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PreMint {
    /// Blinded message
    pub blinded_message: BlindedMessage,
    /// Secret
    pub secret: Secret,
    /// R
    pub r: SecretKey,
    /// Amount
    pub amount: Amount,
}

impl PreMint {
    pub fn generate_for_amount(total_amount: Amount, keyset_id: KeysetId) -> Result<Vec<Self>> {
        total_amount
            .split()
            .into_iter()
            .map(|a| -> Result<_> {
                let secret = Secret::generate();
                let (blinded, r) = blind_message(secret.as_bytes(), None)?;

                let pm = PreMint {
                    blinded_message: BlindedMessage {
                        amount: a,
                        keyset_id,
                        blinded_secret: blinded,
                    },
                    secret,
                    r,
                    amount: total_amount,
                };

                Ok(pm)
            })
            .collect()
    }
}
