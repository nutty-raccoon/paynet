use nuts::{
    Amount,
    traits::{Asset, Unit},
};
use primitive_types::U256;

#[derive(Debug, thiserror::Error)]
pub enum ParseAmountStringError {
    #[error("empty")]
    Empty,
    #[error("empty integer part")]
    EmptyIntegerPart,
    #[error("failed to parse integer part: {0}")]
    IntegerPart(uint::FromDecStrErr),
    #[error("failed to parse fractional part: {0}")]
    FractionalPart(uint::FromDecStrErr),
    #[error("the string cannot contain two periods")]
    MultiplePeriods,
    #[error("amount overflow")]
    Overflow,
    #[error("too many decimals, max {0}")]
    TooManyDecimals(u8),
    #[error("couldn't convert asset amount to unit: {0}")]
    AmountTooBigForU64(&'static str),
    #[error("unit {0} not supported for asset {0}")]
    BadAssetUnitPair(String, String),
}

pub fn parse_asset_amount<A, U>(
    amount_str: &str,
    asset: A,
    unit: U,
) -> Result<Amount, ParseAmountStringError>
where
    A: Asset,
    U: Unit<Asset = A>,
{
    if !unit.is_asset_supported(asset) {
        return Err(ParseAmountStringError::BadAssetUnitPair(
            asset.as_ref().to_string(),
            unit.as_ref().to_string(),
        ));
    }
    if amount_str.is_empty() {
        return Err(ParseAmountStringError::Empty);
    }
    let mut splited_amount_str = amount_str.split('.');

    let integer_part_str = splited_amount_str.next().unwrap();
    if integer_part_str.is_empty() {
        return Err(ParseAmountStringError::EmptyIntegerPart);
    }

    // For STRK/MilliStrk it will be 3 (18 - 15)
    // For Eth/Gwei it will be 9 (18 - 9)
    let scale_order: u8 = asset.precision() - unit.asset_extra_precision();

    // Multiply the the integer part by the 10^scale_order
    let integer_part = U256::from_dec_str(integer_part_str)
        .map_err(ParseAmountStringError::IntegerPart)?
        .checked_mul(U256::from(10).pow(U256::from(scale_order)))
        .ok_or(ParseAmountStringError::Overflow)?;

    let fractional_part = match splited_amount_str.next() {
        None => U256::zero(),
        Some("") => U256::zero(),
        Some(fractional_part_str) => {
            // We cannot accept more digits of precision than made available by scale_order
            // Eg. 3 digits after the period for STRK/MilliStrk
            let scale_factor = if fractional_part_str.len() > usize::from(scale_order) {
                return Err(ParseAmountStringError::TooManyDecimals(scale_order));
            } else {
                // We multiply the fractional part by 10^(scale_order - number of digits)
                U256::from(10).pow(U256::from(
                    (usize::from(scale_order)) - fractional_part_str.len(),
                ))
            };

            U256::from_dec_str(fractional_part_str)
                .map_err(ParseAmountStringError::FractionalPart)?
                .checked_mul(scale_factor)
                .ok_or(ParseAmountStringError::Overflow)?
        }
    };

    // There must only be one period
    if splited_amount_str.next().is_some() {
        return Err(ParseAmountStringError::MultiplePeriods);
    }

    // Combine integer and factorial parts together
    let total_amount = integer_part
        .checked_add(fractional_part)
        .ok_or(ParseAmountStringError::Overflow)?;

    // This will only fail for very big numbers that don't make sense economicaly
    // Nobody is using us to transfer the GDP of a whole country :)
    Ok(Amount::from(
        u64::try_from(total_amount).map_err(ParseAmountStringError::AmountTooBigForU64)?,
    ))
}

#[cfg(test)]
mod parse_asset_amount_test {
    use crate::ParseAmountStringError;

    use super::parse_asset_amount;
    use nuts::Amount;
    use starknet_types::{Asset, Unit};

    #[test]
    fn test_valid_cases() {
        // Basic integer amounts
        assert_eq!(
            parse_asset_amount("1", Asset::Strk, Unit::MilliStrk).unwrap(),
            Amount::from(1_000u64)
        );

        assert_eq!(
            parse_asset_amount("5", Asset::Eth, Unit::Gwei).unwrap(),
            Amount::from(5_000_000_000u64)
        );

        // Decimal amounts
        assert_eq!(
            parse_asset_amount("1.5", Asset::Strk, Unit::MilliStrk).unwrap(),
            Amount::from(1_500u64)
        );

        assert_eq!(
            parse_asset_amount("2.25", Asset::Eth, Unit::Gwei).unwrap(),
            Amount::from(2_250_000_000u64)
        );

        // Zero amounts
        assert_eq!(
            parse_asset_amount("0", Asset::Strk, Unit::MilliStrk).unwrap(),
            Amount::from(0u64)
        );

        assert_eq!(
            parse_asset_amount("0.0", Asset::Eth, Unit::Gwei).unwrap(),
            Amount::from(0u64)
        );
    }

    #[test]
    fn test_leading_and_trailing_zeros() {
        // Leading zeros in integer part
        assert_eq!(
            parse_asset_amount("001", Asset::Strk, Unit::MilliStrk).unwrap(),
            Amount::from(1_000u64)
        );

        assert_eq!(
            parse_asset_amount("0001.5", Asset::Eth, Unit::Gwei).unwrap(),
            Amount::from(1_500_000_000u64)
        );

        // Trailing zeros in fractional part
        assert_eq!(
            parse_asset_amount("1.500", Asset::Strk, Unit::MilliStrk).unwrap(),
            Amount::from(1_500u64)
        );

        assert_eq!(
            parse_asset_amount("2.250000000", Asset::Eth, Unit::Gwei).unwrap(),
            Amount::from(2_250_000_000u64)
        );

        // Both leading and trailing zeros
        assert_eq!(
            parse_asset_amount("00123.450", Asset::Strk, Unit::MilliStrk).unwrap(),
            Amount::from(123_450u64)
        );
    }

    #[test]
    fn test_precision_limits() {
        // STRK with MilliStrk: 3 digits max after decimal (18 - 15 = 3)
        assert!(parse_asset_amount("1.123", Asset::Strk, Unit::MilliStrk).is_ok());

        // ETH with Gwei: 9 digits max after decimal (18 - 9 = 9)
        assert!(parse_asset_amount("1.123456789", Asset::Eth, Unit::Gwei).is_ok());

        // Test exact limits
        assert_eq!(
            parse_asset_amount("1.999", Asset::Strk, Unit::MilliStrk).unwrap(),
            Amount::from(1_999u64)
        );

        assert_eq!(
            parse_asset_amount("1.999999999", Asset::Eth, Unit::Gwei).unwrap(),
            Amount::from(1_999_999_999u64)
        );
    }

    #[test]
    fn test_too_many_decimals() {
        // STRK with MilliStrk: more than 3 digits after decimal
        assert!(matches!(
            parse_asset_amount("1.1234", Asset::Strk, Unit::MilliStrk),
            Err(ParseAmountStringError::TooManyDecimals(3))
        ));

        // ETH with Gwei: more than 9 digits after decimal
        assert!(matches!(
            parse_asset_amount("1.1234567890", Asset::Eth, Unit::Gwei),
            Err(ParseAmountStringError::TooManyDecimals(9))
        ));
    }

    #[test]
    fn test_empty_string() {
        assert!(matches!(
            parse_asset_amount("", Asset::Strk, Unit::MilliStrk),
            Err(ParseAmountStringError::Empty)
        ));

        assert!(matches!(
            parse_asset_amount("", Asset::Eth, Unit::Gwei),
            Err(ParseAmountStringError::Empty)
        ));
    }

    #[test]
    fn test_empty_integer_part() {
        assert!(matches!(
            parse_asset_amount(".5", Asset::Strk, Unit::MilliStrk),
            Err(ParseAmountStringError::EmptyIntegerPart)
        ));

        assert!(matches!(
            parse_asset_amount(".123", Asset::Eth, Unit::Gwei),
            Err(ParseAmountStringError::EmptyIntegerPart)
        ));
    }

    #[test]
    fn test_empty_fractional_part() {
        // Empty fractional part should be treated as zero
        assert_eq!(
            parse_asset_amount("5.", Asset::Strk, Unit::MilliStrk).unwrap(),
            Amount::from(5_000u64)
        );

        assert_eq!(
            parse_asset_amount("10.", Asset::Eth, Unit::Gwei).unwrap(),
            Amount::from(10_000_000_000u64)
        );
    }

    #[test]
    fn test_multiple_periods() {
        assert!(matches!(
            parse_asset_amount("1.2.3", Asset::Strk, Unit::MilliStrk),
            Err(ParseAmountStringError::MultiplePeriods)
        ));

        assert!(matches!(
            parse_asset_amount("1..2", Asset::Eth, Unit::Gwei),
            Err(ParseAmountStringError::MultiplePeriods)
        ));

        assert!(matches!(
            parse_asset_amount("1.2.3.4", Asset::Strk, Unit::MilliStrk),
            Err(ParseAmountStringError::MultiplePeriods)
        ));
    }

    #[test]
    fn test_invalid_characters() {
        // Plus sign
        assert!(matches!(
            parse_asset_amount("+1.5", Asset::Strk, Unit::MilliStrk),
            Err(ParseAmountStringError::IntegerPart(_))
        ));

        // Minus sign
        assert!(matches!(
            parse_asset_amount("-1.5", Asset::Eth, Unit::Gwei),
            Err(ParseAmountStringError::IntegerPart(_))
        ));

        // Scientific notation
        assert!(matches!(
            parse_asset_amount("1e5", Asset::Strk, Unit::MilliStrk),
            Err(ParseAmountStringError::IntegerPart(_))
        ));

        assert!(matches!(
            parse_asset_amount("1.5e2", Asset::Eth, Unit::Gwei),
            Err(ParseAmountStringError::FractionalPart(_))
        ));

        // Hexadecimal
        assert!(matches!(
            parse_asset_amount("0x1A", Asset::Strk, Unit::MilliStrk),
            Err(ParseAmountStringError::IntegerPart(_))
        ));

        assert!(matches!(
            parse_asset_amount("0xFF", Asset::Eth, Unit::Gwei),
            Err(ParseAmountStringError::IntegerPart(_))
        ));

        // Invalid characters in fractional part
        assert!(matches!(
            parse_asset_amount("1.a5", Asset::Strk, Unit::MilliStrk),
            Err(ParseAmountStringError::FractionalPart(_))
        ));

        assert!(matches!(
            parse_asset_amount("1.5x", Asset::Eth, Unit::Gwei),
            Err(ParseAmountStringError::FractionalPart(_))
        ));

        // Spaces
        assert!(matches!(
            parse_asset_amount("1 .5", Asset::Strk, Unit::MilliStrk),
            Err(ParseAmountStringError::IntegerPart(_))
        ));

        assert!(matches!(
            parse_asset_amount("1. 5", Asset::Eth, Unit::Gwei),
            Err(ParseAmountStringError::FractionalPart(_))
        ));
    }

    #[test]
    fn test_overflow_cases() {
        // Large number that will overflow when multiplied by the scale factor
        let very_large_number =
            "115792089237316195423570985008687907853269984665640564039457584007913129639935";
        assert!(matches!(
            parse_asset_amount(very_large_number, Asset::Strk, Unit::MilliStrk),
            Err(ParseAmountStringError::Overflow)
        ));

        assert!(matches!(
            parse_asset_amount(very_large_number, Asset::Eth, Unit::Gwei),
            Err(ParseAmountStringError::Overflow)
        ));

        // Large number that would overflow when adding integer and fractional parts
        let large_int = "9".repeat(50);
        let large_decimal = format!("{}.{}", large_int, "9".repeat(3));
        let x = parse_asset_amount(&large_decimal, Asset::Strk, Unit::MilliStrk);
        assert!(matches!(
            x,
            Err(ParseAmountStringError::AmountTooBigForU64(_))
        ));
    }

    #[test]
    fn test_amount_too_big_for_u64() {
        // This test case depends on the exact implementation and what constitutes
        // "too big for u64" in your context. Here's an example that would likely
        // be too big when converted to the base unit:

        // Maximum u64 is 18,446,744,073,709,551,615
        // For STRK with MilliStrk scale 10^3, max representable value is ~18,446,744,073,709,551 STRK
        // For ETH with Gwei scale 10^9, max representable value is ~18,446,744,073 ETH

        // Test a number that's definitely too big for ETH/Gwei
        assert!(matches!(
            parse_asset_amount("20000000000", Asset::Eth, Unit::Gwei),
            Err(ParseAmountStringError::AmountTooBigForU64(_))
        ));
    }

    #[test]
    fn test_bad_asset_unit_pair() {
        assert!(matches!(
            parse_asset_amount("1.0", Asset::Eth, Unit::MilliStrk),
            Err(ParseAmountStringError::BadAssetUnitPair(_, _))
        ));

        assert!(matches!(
            parse_asset_amount("1.0", Asset::Strk, Unit::Gwei),
            Err(ParseAmountStringError::BadAssetUnitPair(_, _))
        ));
    }

    #[test]
    fn test_edge_case_amounts() {
        // Test very small amounts
        assert_eq!(
            parse_asset_amount("0.001", Asset::Strk, Unit::MilliStrk).unwrap(),
            Amount::from(1u64)
        );

        assert_eq!(
            parse_asset_amount("0.000000001", Asset::Eth, Unit::Gwei).unwrap(),
            Amount::from(1u64)
        );

        // Test amounts with all allowed decimal places
        assert_eq!(
            parse_asset_amount("123.456", Asset::Strk, Unit::MilliStrk).unwrap(),
            Amount::from(123_456u64)
        );

        assert_eq!(
            parse_asset_amount("123.456789012", Asset::Eth, Unit::Gwei).unwrap(),
            Amount::from(123_456_789_012u64)
        );
    }
}
