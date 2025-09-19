use thiserror::Error;

#[cfg(feature = "concurrency")]
#[derive(Debug, Error)]
pub enum ConcurrencyError {
    Melt,
    Mint,
    Swap,
}

#[cfg(feature = "concurrency")]
impl core::fmt::Display for ConcurrencyError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            ConcurrencyError::Melt => write!(f, "Melt"),
            ConcurrencyError::Mint => write!(f, "Mint"),
            ConcurrencyError::Swap => write!(f, "Swap"),
        }
    }
}

#[cfg(feature = "e2e")]
#[derive(Debug, Error)]
pub enum E2eError {
    Melt,
    Mint,
    Receive,
    Send,
}

#[cfg(feature = "e2e")]
impl core::fmt::Display for E2eError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            E2eError::Melt => write!(f, "Melt"),
            E2eError::Mint => write!(f, "Mint"),
            E2eError::Receive => write!(f, "Receive"),
            E2eError::Send => write!(f, "Send"),
        }
    }
}
