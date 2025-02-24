use nuts::{
    Amount,
    dhke::blind_message,
    nut00::secret::Secret,
    nut01::{PublicKey, SecretKey},
};

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PreMint {
    pub amount: Amount,
    pub blinded_secret: PublicKey,
    pub secret: Secret,
    pub r: SecretKey,
}

impl PreMint {
    pub fn generate_for_amount(total_amount: Amount) -> Result<Vec<Self>> {
        total_amount
            .split()
            .into_iter()
            .map(|amount| -> Result<_> {
                let secret = Secret::generate();
                let (blinded_secret, r) = blind_message(secret.as_bytes(), None)?;

                let pm = PreMint {
                    amount,
                    blinded_secret,
                    secret,
                    r,
                };

                Ok(pm)
            })
            .collect()
    }
}
