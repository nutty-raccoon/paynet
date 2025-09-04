#[cfg(test)]
mod tests {
    use crate::Asset;
    use crate::Unit;
    use nuts::Amount;
    use nuts::traits::Asset as AssetT;
    use primitive_types::U256;
    use std::str::FromStr;

    #[test]
    fn test_asset_from_str() {
        assert_eq!(Asset::from_str("strk").unwrap(), Asset::Strk);
        assert_eq!(Asset::from_str("eth").unwrap(), Asset::Eth);
        assert_eq!(Asset::from_str("wbtc").unwrap(), Asset::WBtc);
        assert_eq!(Asset::from_str("usdc").unwrap(), Asset::UsdC);
        assert_eq!(Asset::from_str("usdt").unwrap(), Asset::UsdT);
        assert!(Asset::from_str("invalid").is_err());
    }

    #[test]
    fn test_asset_precision() {
        assert_eq!(Asset::Strk.precision(), 18);
        assert_eq!(Asset::Eth.precision(), 18);
        assert_eq!(Asset::WBtc.precision(), 8);
        assert_eq!(Asset::UsdC.precision(), 6);
        assert_eq!(Asset::UsdT.precision(), 6);
    }

    #[test]
    fn test_asset_scale_factor() {
        assert_eq!(
            Asset::Strk.scale_factor(),
            U256::from(1_000_000_000_000_000_000u64)
        );
        assert_eq!(
            Asset::Eth.scale_factor(),
            U256::from(1_000_000_000_000_000_000u64)
        );
        assert_eq!(Asset::WBtc.scale_factor(), U256::from(100_000_000u64));
        assert_eq!(Asset::UsdC.scale_factor(), U256::from(1_000_000u64));
        assert_eq!(Asset::UsdT.scale_factor(), U256::from(1_000_000u64));
    }

    #[test]
    fn test_asset_find_best_unit() {
        assert_eq!(Asset::Strk.find_best_unit(), Unit::MilliStrk);
        assert_eq!(Asset::Eth.find_best_unit(), Unit::Gwei);
        assert_eq!(Asset::WBtc.find_best_unit(), Unit::Satoshi);
        assert_eq!(Asset::UsdC.find_best_unit(), Unit::CentiUsdC);
        assert_eq!(Asset::UsdT.find_best_unit(), Unit::CentiUsdT);
    }

    #[test]
    fn test_asset_conversions() {
        // Test STRK conversion
        let strk_amount = U256::from(1_000_000_000_000_000_000u64); // 1 STRK
        let (amount, unit, rem) = Asset::Strk.convert_to_amount_and_unit(strk_amount).unwrap();
        assert_eq!(amount, Amount::from(1000u64)); // Should be 1000 milliSTRK
        assert_eq!(unit, Unit::MilliStrk);
        assert_eq!(rem, U256::zero());

        // Test ETH conversion
        let eth_amount = U256::from(1_000_000_000_000_000_000u64); // 1 ETH
        let (amount, unit, rem) = Asset::Eth.convert_to_amount_and_unit(eth_amount).unwrap();
        assert_eq!(amount, Amount::from(1_000_000_000u64)); // Should be 1,000,000,000 Gwei
        assert_eq!(unit, Unit::Gwei);
        assert_eq!(rem, U256::zero());

        // Test WBTC conversion
        let wbtc_amount = U256::from(100_000_000u64); // 1 WBTC
        let (amount, unit, rem) = Asset::WBtc.convert_to_amount_and_unit(wbtc_amount).unwrap();
        assert_eq!(amount, Amount::from(100_000_000u64)); // Should be 100,000,000 satoshis
        assert_eq!(unit, Unit::Satoshi);
        assert_eq!(rem, U256::zero());

        // Test USDC conversion
        let usdc_amount = U256::from(1_000_000u64); // 1 USDC
        let (amount, unit, rem) = Asset::UsdC.convert_to_amount_and_unit(usdc_amount).unwrap();
        assert_eq!(amount, Amount::from(100u64)); // Should be 100 cents
        assert_eq!(unit, Unit::CentiUsdC);
        assert_eq!(rem, U256::zero());

        // Test USDT conversion
        let usdt_amount = U256::from(1_000_000u64); // 1 USDC
        let (amount, unit, rem) = Asset::UsdT.convert_to_amount_and_unit(usdt_amount).unwrap();
        assert_eq!(amount, Amount::from(100u64)); // Should be 100 cents
        assert_eq!(unit, Unit::CentiUsdT);
        assert_eq!(rem, U256::zero());
    }
}
