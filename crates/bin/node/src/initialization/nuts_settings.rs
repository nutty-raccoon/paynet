use nuts::{Amount, nut04::MintMethodSettings, nut05::MeltMethodSettings, nut06::NutsSettings};
use starknet_types::Unit;

use crate::methods::Method;

// TODO: make it a compile time const
pub(super) fn nuts_settings() -> NutsSettings<Method, Unit> {
    NutsSettings {
        nut04: nuts::nut04::Settings {
            methods: vec![MintMethodSettings {
                method: Method::Starknet,
                unit: Unit::Strk,
                min_amount: Some(Amount::ONE),
                max_amount: None,
                description: true,
            }],
            disabled: false,
        },
        nut05: nuts::nut05::Settings {
            methods: vec![MeltMethodSettings {
                method: Method::Starknet,
                unit: Unit::Strk,
                min_amount: Some(Amount::ONE),
                max_amount: None,
            }],
            disabled: false,
        },
    }
}
