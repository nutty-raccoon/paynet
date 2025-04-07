use bitcoin_hashes::Sha256;
use num_bigint::BigUint;
use nuts::Amount;
use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StarknetU256 {
    pub low: Felt,
    pub high: Felt,
}

impl StarknetU256 {
    pub const ZERO: StarknetU256 = StarknetU256 {
        low: Felt::ZERO,
        high: Felt::ZERO,
    };
}

#[derive(Debug, thiserror::Error)]
#[error("Slice too long, max is 32, received {0}")]
pub struct StarknetU256FromBytesSliceError(usize);

impl StarknetU256 {
    pub fn from_parts<L: Into<u128>, H: Into<u128>>(low: L, high: H) -> Self {
        let low: u128 = low.into();
        let high: u128 = high.into();
        Self {
            low: Felt::from(low),
            high: Felt::from(high),
        }
    }

    pub fn to_bytes_be(&self) -> [u8; 32] {
        let mut ret = self.low.to_bytes_be();

        ret[0..16].copy_from_slice(&self.high.to_bytes_be()[16..32]);

        ret
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self {
            low: Felt::from(u128::from_be_bytes(bytes[16..].try_into().unwrap())),
            high: Felt::from(u128::from_be_bytes(bytes[..16].try_into().unwrap())),
        }
    }

    pub fn from_bytes_slice(bytes: &[u8]) -> Result<Self, StarknetU256FromBytesSliceError> {
        let (low, high) = match bytes.len() {
            0 => return Ok(Self::ZERO),
            16 => (u128::from_be_bytes(bytes.try_into().unwrap()), 0u128),
            32 => (
                u128::from_be_bytes(bytes[16..32].try_into().unwrap()),
                u128::from_be_bytes(bytes[0..16].try_into().unwrap()),
            ),
            l if l < 16 => {
                let mut low = [0u8; 16];
                low[16 - l..].copy_from_slice(bytes);
                (u128::from_be_bytes(low), 0u128)
            }
            l if l < 32 => {
                let mut low = [0u8; 16];
                let mut high = [0u8; 16];
                low.copy_from_slice(&bytes[l - 16..]);
                high.copy_from_slice(&bytes[0..l - 16]);
                (u128::from_be_bytes(low), u128::from_be_bytes(high))
            }
            l => return Err(StarknetU256FromBytesSliceError(l)),
        };

        Ok(Self::from_parts(low, high))
    }
}

impl From<Sha256> for StarknetU256 {
    fn from(value: Sha256) -> Self {
        let bytes = value.as_byte_array();
        StarknetU256::from_bytes(bytes)
    }
}

impl core::fmt::Display for StarknetU256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "low: {:#x} - high: {:#x}", self.low, self.high)
    }
}

impl From<Amount> for StarknetU256 {
    fn from(value: Amount) -> Self {
        Self {
            low: value.into(),
            high: Felt::ZERO,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TryU256FromBigUintError {
    #[error("BigUint too big")]
    TooBig,
}

impl TryFrom<BigUint> for StarknetU256 {
    type Error = TryU256FromBigUintError;

    fn try_from(value: BigUint) -> Result<Self, Self::Error> {
        let bytes = value.to_bytes_le();
        if bytes.len() > 32 {
            return Err(Self::Error::TooBig);
        };

        if bytes.len() < 16 {
            return Ok(StarknetU256 {
                low: Felt::from_bytes_le_slice(&bytes),
                high: Felt::ZERO,
            });
        }

        Ok(StarknetU256 {
            low: Felt::from_bytes_le_slice(&bytes[0..16]),
            high: Felt::from_bytes_le_slice(&bytes[16..]),
        })
    }
}

impl From<primitive_types::U256> for StarknetU256 {
    fn from(value: primitive_types::U256) -> Self {
        let bytes = value.to_little_endian();
        let low = u128::from_le_bytes(bytes[..16].try_into().unwrap());
        let high = u128::from_le_bytes(bytes[16..].try_into().unwrap());
        Self {
            low: Felt::from(low),
            high: Felt::from(high),
        }
    }
}

impl From<StarknetU256> for primitive_types::U256 {
    fn from(value: StarknetU256) -> Self {
        Self::from(&value)
    }
}

impl From<&StarknetU256> for primitive_types::U256 {
    fn from(value: &StarknetU256) -> Self {
        let mut bytes = value.low.to_bytes_le();
        bytes[16..].copy_from_slice(&value.high.to_bytes_le()[..16]);

        primitive_types::U256::from_little_endian(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use starknet_types_core::felt::Felt;

    use super::StarknetU256;

    #[test]
    fn starknet_and_primitive_types_u256_conversion() {
        let pt = primitive_types::U256::MAX;
        let s = StarknetU256::from(pt);

        assert_eq!(primitive_types::U256::from(s), pt);

        let pt = primitive_types::U256::zero();
        let s = StarknetU256::from(pt);

        assert_eq!(primitive_types::U256::from(s), pt);

        let pt = primitive_types::U256::one();
        let s = StarknetU256::from(pt);

        assert_eq!(primitive_types::U256::from(s), pt);

        let s = StarknetU256 {
            low: Felt::from_hex_unchecked("0xbabe"),
            high: Felt::from_hex_unchecked("0xcafe"),
        };
        let pt = primitive_types::U256::from(&s);

        assert_eq!(StarknetU256::from(pt), s);
    }
}
