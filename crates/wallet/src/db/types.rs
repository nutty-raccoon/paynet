use rusqlite::{
    Result,
    types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
};

use crate::types::{Amount, NodeUrl, ProofState, Secret};

// NodeUrl implementation
impl ToSql for NodeUrl {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok(self.as_ref().into())
    }
}

impl FromSql for NodeUrl {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        String::column_result(value).map(|s| NodeUrl::new_unchecked(s))
    }
}

// ProofState implementation
impl ToSql for ProofState {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok((*self as u8).into())
    }
}

impl FromSql for ProofState {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        u8::column_result(value).and_then(|v| match v {
            1 => Ok(ProofState::Unspent),
            2 => Ok(ProofState::Pending),
            3 => Ok(ProofState::Spent),
            4 => Ok(ProofState::Reserved),
            v => Err(FromSqlError::OutOfRange(v.into())),
        })
    }
}

// KeysetId implementation
impl ToSql for [u8; 8] {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok(self.as_ref().into())
    }
}

impl FromSql for [u8; 8] {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let bytes = <Vec<u8> as FromSql>::column_result(value)?;
        bytes.try_into().map_err(|_| FromSqlError::InvalidType)
    }
}

// PublicKey implementation
impl ToSql for [u8; 33] {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok(self.as_ref().into())
    }
}

impl FromSql for [u8; 33] {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let bytes = <Vec<u8> as FromSql>::column_result(value)?;
        bytes.try_into().map_err(|_| FromSqlError::InvalidType)
    }
}

// Amount implementation
impl ToSql for Amount {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok(u64::from(*self).into())
    }
}

impl FromSql for Amount {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        u64::column_result(value).map(Amount::from)
    }
}

// Secret implementation
impl ToSql for Secret {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok(self.to_string().into())
    }
}

impl FromSql for Secret {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        String::column_result(value)
            .and_then(|s| Secret::new(&s).map_err(|_| FromSqlError::InvalidType))
    }
}

// MintQuoteState implementation
#[derive(Debug, Clone, Copy)]
pub enum MintQuoteState {
    Pending = 1,
    Paid = 2,
    Expired = 3,
}

impl ToSql for MintQuoteState {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok((*self as u8).into())
    }
}

impl FromSql for MintQuoteState {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        u8::column_result(value).and_then(|v| match v {
            1 => Ok(MintQuoteState::Pending),
            2 => Ok(MintQuoteState::Paid),
            3 => Ok(MintQuoteState::Expired),
            v => Err(FromSqlError::OutOfRange(v.into())),
        })
    }
}

// MeltResponseState implementation
#[derive(Debug, Clone, Copy)]
pub enum MeltResponseState {
    Pending = 1,
    Paid = 2,
    Expired = 3,
}

impl ToSql for MeltResponseState {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok((*self as u8).into())
    }
}

impl FromSql for MeltResponseState {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        u8::column_result(value).and_then(|v| match v {
            1 => Ok(MeltResponseState::Pending),
            2 => Ok(MeltResponseState::Paid),
            3 => Ok(MeltResponseState::Expired),
            v => Err(FromSqlError::OutOfRange(v.into())),
        })
    }
}
