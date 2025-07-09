use bitcoin::bip32::Xpriv;
use nuts::{
    dhke::blind_message, nut00::{self, secret::Secret}, nut01::{PublicKey, SecretKey}, nut02::KeysetId, Amount, SplitTarget
};

use rusqlite::{
    ToSql,
    types::{FromSql, FromSqlError},
};

use crate::errors::Error;
mod node_url;
pub use node_url::{Error as NodeUrlError, NodeUrl};
pub mod compact_wad;

#[derive(Debug, Clone)]
pub struct PreMint {
    pub amount: Amount,
    pub blinded_secret: PublicKey,
    pub secret: Secret,
    pub r: SecretKey,
}

impl PreMint {
    pub fn generate_for_amount(
        xpriv: Xpriv,
        keyset_id: KeysetId,
        keyset_counter: u32,
        total_amount: Amount,
        split_target: &SplitTarget,
    ) -> Result<Vec<Self>, Error> {
        total_amount
            .split_targeted(split_target)?
            .into_iter()
            .map(|amount| -> Result<_, Error> {
                let secret = Secret::from_xpriv(xpriv, keyset_id, keyset_counter).unwrap();
                let blinding_factor = SecretKey::from_xpriv(xpriv, keyset_id, keyset_counter).unwrap();

                let (blinded_secret, r) = blind_message(secret.as_bytes(), Some(blinding_factor))?;

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

#[derive(Debug, Clone, Copy)]
pub enum ProofState {
    Unspent = 1,
    Pending = 2,
    Spent = 3,
    Reserved = 4,
}

impl ToSql for ProofState {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok((*self as u8).into())
    }
}

impl FromSql for ProofState {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        u8::column_result(value).and_then(|v| match v {
            1 => Ok(ProofState::Unspent),
            2 => Ok(ProofState::Pending),
            3 => Ok(ProofState::Spent),
            4 => Ok(ProofState::Reserved),
            v => Err(FromSqlError::OutOfRange(v.into())),
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Wad {
    pub node_url: NodeUrl,
    pub proofs: Vec<nut00::Proof>,
}
