use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    Amount,
    nut00::{BlindSignature, BlindedMessage},
    traits::Unit,
};
use rusqlite::{
    Result,
    types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unknown state")]
    UnknownState,
}

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "UPPERCASE")]
#[cfg_attr(
    feature = "sqlx",
    derive(sqlx::Type),
    sqlx(type_name = "mint_quote_state", rename_all = "UPPERCASE")
)]
pub enum MintQuoteState {
    /// Quote has not been paid
    #[default]
    Unpaid,
    /// Quote has been paid and wallet can mint
    Paid,
    /// ecash issued for quote
    Issued,
}

impl core::fmt::Display for MintQuoteState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MintQuoteState::Unpaid => "UNPAID",
                MintQuoteState::Paid => "PAID",
                MintQuoteState::Issued => "ISSUED",
            }
        )
    }
}

impl ToSql for MintQuoteState {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok(ToSqlOutput::from(self.to_string()))
    }
}

impl FromSql for MintQuoteState {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| match s {
            "UNPAID" => Ok(MintQuoteState::Unpaid),
            "PAID" => Ok(MintQuoteState::Paid),
            "ISSUED" => Ok(MintQuoteState::Issued),
            _ => Err(FromSqlError::Other(Box::new(Error::UnknownState))),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MintQuoteRequest<U: Unit> {
    pub amount: Amount,
    pub unit: U,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MintQuoteResponse<Q> {
    pub quote: Q,
    pub request: String,
    pub state: MintQuoteState,
    pub expiry: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MintRequest<Q> {
    pub quote: Q,
    pub outputs: Vec<BlindedMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MintResponse {
    pub signatures: Vec<BlindSignature>,
}

/// Mint Method Settings
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct MintMethodSettings<M, U> {
    /// Payment Method e.g. Starknet
    pub method: M,
    /// Currency Unit e.g. strk
    pub unit: U,
    /// Min Amount
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_amount: Option<Amount>,
    /// Max Amount
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_amount: Option<Amount>,
    /// Quote Description
    #[serde(default)]
    pub description: bool,
}

/// Mint Settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings<M, U> {
    /// Methods to mint
    pub methods: Vec<MintMethodSettings<M, U>>,
    /// Minting disabled
    pub disabled: bool,
}

impl<M, U> Settings<M, U>
where
    M: PartialEq + Eq + Clone,
    U: PartialEq + Eq + Clone,
{
    pub fn get_settings(&self, method: M, unit: U) -> Option<MintMethodSettings<M, U>> {
        self.methods
            .iter()
            .find(|&s| method == s.method && unit == s.unit)
            .cloned()
    }
}
