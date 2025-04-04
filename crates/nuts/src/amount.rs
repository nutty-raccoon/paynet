//! CDK Amount
//!
//! Is any unit and will be treated as the unit of the wallet

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use num_traits::{CheckedAdd, CheckedSub, One, Zero};
#[cfg(feature = "rusqlite")]
use rusqlite::{
    Result,
    types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// Amount Error
#[derive(Debug, Error)]
pub enum Error {
    /// Split Values must be less then or equal to amount
    #[error("Split Values must be less then or equal to amount")]
    SplitValuesGreater,
    /// Amount overflow
    #[error("Amount Overflow")]
    AmountOverflow,
    /// Cannot convert units
    #[error("Cannot convert units")]
    CannotConvertUnits,
}

/// Amount can be any unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Amount(u64);

impl Amount {
    /// Amount zero
    pub const ZERO: Amount = Amount(0);
    pub const ONE: Amount = Amount(1);

    /// Split into parts that are powers of two
    pub fn split(&self) -> Vec<Self> {
        let sats = self.0;
        (0_u64..64)
            .rev()
            .filter_map(|bit| {
                let part = 1 << bit;
                ((sats & part) == part).then_some(Self::from(part))
            })
            .collect()
    }

    /// Split into parts that are powers of two by target
    pub fn split_targeted(&self, target: &SplitTarget) -> Result<Vec<Self>, Error> {
        let mut parts = match target {
            SplitTarget::None => self.split(),
            SplitTarget::Value(amount) => {
                if self.le(amount) {
                    return Ok(self.split());
                }

                let mut parts_total = Amount::ZERO;
                let mut parts = Vec::new();

                // The powers of two that are need to create target value
                let parts_of_value = amount.split();

                while parts_total.lt(self) {
                    for part in parts_of_value.iter().copied() {
                        if (part + parts_total).le(self) {
                            parts.push(part);
                        } else {
                            let amount_left = *self - parts_total;
                            parts.extend(amount_left.split());
                        }

                        parts_total = Amount::try_sum(parts.clone().iter().copied())?;

                        if parts_total.eq(self) {
                            break;
                        }
                    }
                }

                parts
            }
            SplitTarget::Values(values) => {
                let values_total: Amount = Amount::try_sum(values.clone().into_iter())?;

                match self.cmp(&values_total) {
                    Ordering::Equal => values.clone(),
                    Ordering::Less => {
                        return Err(Error::SplitValuesGreater);
                    }
                    Ordering::Greater => {
                        let extra = *self - values_total;
                        let mut extra_amount = extra.split();
                        let mut values = values.clone();

                        values.append(&mut extra_amount);
                        values
                    }
                }
            }
        };

        parts.sort();
        Ok(parts)
    }

    /// Try sum to check for overflow
    pub fn try_sum<I>(iter: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = Self>,
    {
        iter.into_iter().try_fold(Amount::ZERO, |acc, x| {
            acc.checked_add(&x).ok_or(Error::AmountOverflow)
        })
    }

    pub fn into_i64_repr(&self) -> i64 {
        i64::from_be_bytes(self.0.to_be_bytes())
    }

    pub fn from_i64_repr(value: i64) -> Self {
        Self(u64::from_be_bytes(value.to_be_bytes()))
    }
}

impl Zero for Amount {
    fn zero() -> Self {
        Self(0)
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl One for Amount {
    fn one() -> Self {
        Self(1)
    }
}

impl CheckedAdd for Amount {
    fn checked_add(&self, other: &Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Amount)
    }
}
impl CheckedSub for Amount {
    fn checked_sub(&self, other: &Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Amount)
    }
}

impl Default for Amount {
    fn default() -> Self {
        Amount::ZERO
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(width) = f.width() {
            write!(f, "{:width$}", self.0, width = width)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl From<u16> for Amount {
    fn from(value: u16) -> Self {
        Self(value.into())
    }
}

impl From<u64> for Amount {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<&u64> for Amount {
    fn from(value: &u64) -> Self {
        Self(*value)
    }
}

impl From<Amount> for u64 {
    fn from(value: Amount) -> Self {
        value.0
    }
}

impl std::ops::Add for Amount {
    type Output = Amount;

    fn add(self, rhs: Amount) -> Self::Output {
        Amount(self.0.checked_add(rhs.0).expect("Addition error"))
    }
}

impl std::ops::AddAssign for Amount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.checked_add(rhs.0).expect("Addition error");
    }
}

impl std::ops::Sub for Amount {
    type Output = Amount;

    fn sub(self, rhs: Amount) -> Self::Output {
        Amount(self.0 - rhs.0)
    }
}

impl std::ops::SubAssign for Amount {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl std::ops::Mul for Amount {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Amount(self.0 * other.0)
    }
}

impl std::ops::Div for Amount {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        Amount(self.0 / other.0)
    }
}

#[cfg(feature = "starknet")]
impl From<Amount> for starknet_types_core::felt::Felt {
    fn from(value: Amount) -> Self {
        value.0.into()
    }
}

impl From<Amount> for num_bigint::BigUint {
    fn from(value: Amount) -> Self {
        Self::from(value.0)
    }
}

/// String wrapper for an [Amount].
///
/// It ser-/deserializes the inner [Amount] to a string, while at the same time using the [u64]
/// value of the [Amount] for comparison and ordering. This helps automatically sort the keys of
/// a [BTreeMap] when [AmountStr] is used as key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmountStr(Amount);

impl AmountStr {
    pub(crate) fn from(amt: Amount) -> Self {
        Self(amt)
    }
}

impl PartialOrd<Self> for AmountStr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AmountStr {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<'de> Deserialize<'de> for AmountStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        u64::from_str(&s)
            .map(Amount)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for AmountStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

/// Kinds of targeting that are supported
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub enum SplitTarget {
    /// Default target; least amount of proofs
    #[default]
    None,
    /// Target amount for wallet to have most proofs that add up to value
    Value(Amount),
    /// Specific amounts to split into **MUST** equal amount being split
    Values(Vec<Amount>),
}

#[cfg(feature = "rusqlite")]
impl ToSql for Amount {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

#[cfg(feature = "rusqlite")]
impl FromSql for Amount {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        u64::column_result(value).map(Amount::from)
    }
}

#[cfg(test)]
mod tests {
    /// Msats in sat
    pub const MSAT_IN_SAT: u64 = 1000;

    /// Helper function to convert units
    pub fn to_unit<T>(
        amount: T,
        current_unit: &TestUnit,
        target_unit: &TestUnit,
    ) -> Result<Amount, Error>
    where
        T: Into<u64>,
    {
        let amount = amount.into();
        match (current_unit, target_unit) {
            (TestUnit::Sat, TestUnit::Sat) => Ok(amount.into()),
            (TestUnit::Msat, TestUnit::Msat) => Ok(amount.into()),
            (TestUnit::Sat, TestUnit::Msat) => Ok((amount * MSAT_IN_SAT).into()),
            (TestUnit::Msat, TestUnit::Sat) => Ok((amount / MSAT_IN_SAT).into()),
            (TestUnit::Usd, TestUnit::Usd) => Ok(amount.into()),
            (TestUnit::Eur, TestUnit::Eur) => Ok(amount.into()),
            _ => Err(Error::CannotConvertUnits),
        }
    }

    use crate::traits::test_types::TestUnit;

    use super::*;

    #[test]
    fn test_split_amount() {
        assert_eq!(Amount(1).split(), vec![Amount(1)]);
        assert_eq!(Amount(2).split(), vec![Amount(2)]);
        assert_eq!(Amount(3).split(), vec![Amount(2), Amount(1)]);
        let amounts: Vec<Amount> = [8, 2, 1].iter().map(|a| Amount(*a)).collect();
        assert_eq!(Amount(11).split(), amounts);
        let amounts: Vec<Amount> = [128, 64, 32, 16, 8, 4, 2, 1]
            .iter()
            .map(|a| Amount(*a))
            .collect();
        assert_eq!(Amount(255).split(), amounts);
    }

    #[test]
    fn test_split_target_amount() {
        let amount = Amount(65);

        let split = amount
            .split_targeted(&SplitTarget::Value(Amount(32)))
            .unwrap();
        assert_eq!(vec![Amount(1), Amount(32), Amount(32)], split);

        let amount = Amount(150);

        let split = amount
            .split_targeted(&SplitTarget::Value(Amount(50)))
            .unwrap();
        assert_eq!(
            vec![
                Amount(2),
                Amount(2),
                Amount(2),
                Amount(16),
                Amount(16),
                Amount(16),
                Amount(32),
                Amount(32),
                Amount(32)
            ],
            split
        );

        let amount = Amount(63);

        let split = amount
            .split_targeted(&SplitTarget::Value(Amount(32)))
            .unwrap();
        assert_eq!(
            vec![
                Amount(1),
                Amount(2),
                Amount(4),
                Amount(8),
                Amount(16),
                Amount(32)
            ],
            split
        );
    }

    #[test]
    fn test_split_values() {
        let amount = Amount(10);

        let target = vec![Amount(2), Amount(4), Amount(4)];

        let split_target = SplitTarget::Values(target.clone());

        let values = amount.split_targeted(&split_target).unwrap();

        assert_eq!(target, values);

        let target = vec![Amount(2), Amount(4), Amount(4)];

        let split_target = SplitTarget::Values(vec![Amount(2), Amount(4)]);

        let values = amount.split_targeted(&split_target).unwrap();

        assert_eq!(target, values);

        let split_target = SplitTarget::Values(vec![Amount(2), Amount(10)]);

        let values = amount.split_targeted(&split_target);

        assert!(values.is_err())
    }

    #[test]
    #[should_panic]
    fn test_amount_addition() {
        let amount_one: Amount = u64::MAX.into();
        let amount_two = Amount(1);

        let amounts = vec![amount_one, amount_two];

        let _total: Amount = Amount::try_sum(amounts).unwrap();
    }

    #[test]
    fn test_try_amount_addition() {
        let amount_one: Amount = u64::MAX.into();
        let amount_two = Amount(1);

        let amounts = vec![amount_one, amount_two];

        let total = Amount::try_sum(amounts);

        assert!(total.is_err());
        let amount_one = Amount(10000);
        let amount_two = Amount(1);

        let amounts = vec![amount_one, amount_two];
        let total = Amount::try_sum(amounts).unwrap();

        assert_eq!(total, Amount(10001));
    }

    #[test]
    fn test_amount_to_unit() {
        let amount = Amount(1000);
        let current_unit = TestUnit::Sat;
        let target_unit = TestUnit::Msat;

        let converted = to_unit(amount, &current_unit, &target_unit).unwrap();

        assert_eq!(converted, Amount(1000000));

        let amount = Amount(1000);
        let current_unit = TestUnit::Msat;
        let target_unit = TestUnit::Sat;

        let converted = to_unit(amount, &current_unit, &target_unit).unwrap();

        assert_eq!(converted, Amount(1));

        let amount = Amount(1);
        let current_unit = TestUnit::Usd;
        let target_unit = TestUnit::Usd;

        let converted = to_unit(amount, &current_unit, &target_unit).unwrap();

        assert_eq!(converted, Amount(1));

        let amount = Amount(1);
        let current_unit = TestUnit::Eur;
        let target_unit = TestUnit::Eur;

        let converted = to_unit(amount, &current_unit, &target_unit).unwrap();

        assert_eq!(converted, Amount(1));

        let amount = Amount(1);
        let current_unit = TestUnit::Sat;
        let target_unit = TestUnit::Eur;

        let converted = to_unit(amount, &current_unit, &target_unit);

        assert!(converted.is_err());
    }
}
