use bitcoin::bip32::Xpriv;
use node_client::{BlindSignature, BlindedMessage};
use nuts::{
    Amount, SplitTarget,
    dhke::blind_message,
    nut00::{self, secret::Secret},
    nut01::{PublicKey, SecretKey},
    nut02::KeysetId,
};

use rusqlite::{
    Connection, ToSql, Transaction,
    types::{FromSql, FromSqlError},
};

use crate::{
    db::{self, wallet},
    errors::Error,
    get_active_keyset_for_unit, store_new_proofs_from_blind_signatures,
};
mod node_url;
pub use node_url::{Error as NodeUrlError, NodeUrl};
pub mod compact_wad;

#[derive(Debug)]
pub struct BlindingData {
    xpriv: Xpriv,
    keyset_id: KeysetId,
    keyset_counter: u32,
}

impl BlindingData {
    pub fn load_from_db(db_conn: &Connection, node_id: u32, unit: &str) -> Result<Self, Error> {
        let (id, counter) = get_active_keyset_for_unit(db_conn, node_id, unit)?;
        let pk = wallet::get_private_key(db_conn)?.unwrap();

        Ok(Self {
            xpriv: pk,
            keyset_id: id,
            keyset_counter: counter,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PreMint {
    pub amount: Amount,
    pub blinded_secret: PublicKey,
    pub secret: Secret,
    pub r: SecretKey,
}

pub struct PreMints {
    keyset_id: KeysetId,
    initial_keyset_counter: u32,
    pre_mints: Vec<PreMint>,
}

impl PreMints {
    pub fn generate_for_amount(
        total_amount: Amount,
        split_target: &SplitTarget,
        blinding_data: BlindingData,
    ) -> Result<Self, Error> {
        let pre_mints = total_amount
            .split_targeted(split_target)?
            .into_iter()
            .enumerate()
            .map(|(i, amount)| -> Result<_, Error> {
                let secret = Secret::from_xpriv(
                    blinding_data.xpriv,
                    blinding_data.keyset_id,
                    blinding_data.keyset_counter + i as u32,
                )?;
                let blinding_factor = SecretKey::from_xpriv(
                    blinding_data.xpriv,
                    blinding_data.keyset_id,
                    blinding_data.keyset_counter + i as u32,
                )?;

                let (blinded_secret, r) = blind_message(&secret.to_bytes(), Some(blinding_factor))?;

                let pm = PreMint {
                    amount,
                    blinded_secret,
                    secret,
                    r,
                };

                Ok(pm)
            })
            .collect::<Result<Vec<PreMint>, _>>()?;

        Ok(PreMints {
            keyset_id: blinding_data.keyset_id,
            initial_keyset_counter: blinding_data.keyset_counter,
            pre_mints,
        })
    }

    pub fn build_node_client_outputs(&self) -> Vec<BlindedMessage> {
        self.pre_mints
            .iter()
            .map(|pm| node_client::BlindedMessage {
                amount: pm.amount.into(),
                keyset_id: self.keyset_id.to_bytes().to_vec(),
                blinded_secret: pm.blinded_secret.to_bytes().to_vec(),
            })
            .collect()
    }

    pub fn store_new_tokens(
        self,
        tx: &Transaction,
        node_id: u32,
        signatures: Vec<BlindSignature>,
    ) -> Result<Vec<(PublicKey, Amount)>, Error> {
        db::keyset::set_counter(
            tx,
            self.keyset_id,
            self.initial_keyset_counter + self.pre_mints.len() as u32,
        )?;
        let signatures_iterator = self.pre_mints.into_iter().zip(signatures).map(
            |(pm, bs)| -> Result<_, nuts::nut01::Error> {
                Ok((
                    PublicKey::from_slice(&bs.blind_signature)?,
                    pm.secret,
                    pm.r,
                    pm.amount,
                ))
            },
        );

        let new_tokens = store_new_proofs_from_blind_signatures(
            tx,
            node_id,
            self.keyset_id,
            signatures_iterator,
        )?;

        Ok(new_tokens)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
