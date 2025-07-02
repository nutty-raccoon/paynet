//! NUT-05: Melting Tokens

use crate::{Amount, nut00::Proofs, traits::Unit};
#[cfg(feature = "rusqlite")]
use rusqlite::{
    Result,
    types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// NUT05 Error
#[derive(Debug, Error)]
pub enum Error {
    /// Unknown Quote State
    #[error("Unknown quote state")]
    UnknownState,
    /// Amount overflow
    #[error("Amount Overflow")]
    AmountOverflow,
}

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "UPPERCASE")]
#[cfg_attr(
    feature = "sqlx",
    derive(sqlx::Type),
    sqlx(type_name = "melt_quote_state", rename_all = "UPPERCASE")
)]
pub enum MeltQuoteState {
    /// Quote has not been paid
    #[default]
    Unpaid,
    /// on-chain payment is being done
    Pending,
    /// Payment has been done on chain
    Paid,
}

impl From<MeltQuoteState> for i32 {
    fn from(state: MeltQuoteState) -> Self {
        match state {
            MeltQuoteState::Unpaid => 1,
            MeltQuoteState::Pending => 2,
            MeltQuoteState::Paid => 3,
        }
    }
}

impl core::fmt::Display for MeltQuoteState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MeltQuoteState::Unpaid => "UNPAID",
                MeltQuoteState::Pending => "PENDING",
                MeltQuoteState::Paid => "PAID",
            }
        )
    }
}

#[cfg(feature = "rusqlite")]
impl ToSql for MeltQuoteState {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok(ToSqlOutput::from(match self {
            MeltQuoteState::Unpaid => 1,
            MeltQuoteState::Pending => 2,
            MeltQuoteState::Paid => 3,
        }))
    }
}

#[cfg(feature = "rusqlite")]
impl FromSql for MeltQuoteState {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_i64().and_then(|i| match i {
            1 => Ok(MeltQuoteState::Unpaid),
            2 => Ok(MeltQuoteState::Pending),
            3 => Ok(MeltQuoteState::Paid),
            _ => Err(FromSqlError::Other(Box::new(Error::UnknownState))),
        })
    }
}

/// Melt quote request [NUT-05]
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeltQuoteRequest<U: Unit> {
    /// Invoice to be paid
    pub request: String,
    /// Unit wallet would like to pay with
    pub unit: U,
}

/// Melt quote response [NUT-05]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MeltQuoteResponse<Q, U: Unit> {
    /// Quote Id
    pub quote: Q,
    /// The amount that needs to be provided
    pub amount: Amount,
    /// The unit that needs to be provided
    pub unit: U,
    /// Quote State
    pub state: MeltQuoteState,
    /// Unix timestamp until the quote is valid
    pub expiry: u64,
}

/// Melt Request [NUT-05]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeltRequest<Q> {
    /// Quote ID
    pub quote: Q,
    /// Proofs
    pub inputs: Proofs,
}

/// Melt Method Settings
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MeltMethodSettings<M, U> {
    /// Payment Method e.g. bolt11
    pub method: M,
    /// Currency Unit e.g. sat
    pub unit: U,
    /// Min Amount
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_amount: Option<Amount>,
    /// Max Amount
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_amount: Option<Amount>,
}

impl<M, U> Settings<M, U>
where
    M: PartialEq + Eq + Clone,
    U: PartialEq + Eq + Clone,
{
    pub fn get_settings(&self, method: M, unit: U) -> Option<MeltMethodSettings<M, U>> {
        self.methods
            .iter()
            .find(|&s| method == s.method && unit == s.unit)
            .cloned()
    }
}

/// Melt Settings
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Settings<M, U> {
    /// Methods to melt
    pub methods: Vec<MeltMethodSettings<M, U>>,
    /// Minting disabled
    pub disabled: bool,
}
