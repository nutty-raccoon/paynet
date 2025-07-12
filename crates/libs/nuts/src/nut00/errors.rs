use thiserror::Error;

/// NUT00 Error
#[derive(Debug, Error)]
pub enum Error {
    /// Proofs required
    #[error("Proofs required in token")]
    ProofsRequired,
    /// Unsupported token
    #[error("Unsupported token")]
    UnsupportedToken,
    /// Unsupported token
    #[error("Unsupported unit")]
    UnsupportedUnit,
    /// Unsupported token
    #[error("Unsupported payment method")]
    UnsupportedPaymentMethod,
    // /// Serde Json error
    // #[error(transparent)]
    // SerdeJsonError(#[from] serde_json::Error),
    // /// Utf8 parse error
    // #[error(transparent)]
    // Utf8ParseError(#[from] FromUtf8Error),
    /// Base64 error
    #[error(transparent)]
    Base64(#[from] bitcoin::base64::DecodeError),
    /// Ciborium error
    #[error(transparent)]
    Ciborium(#[from] ciborium::de::Error<std::io::Error>),
    // /// Amount Error
    // #[error(transparent)]
    // Amount(#[from] crate::amount::Error),
    // /// Secret error
    // #[error(transparent)]
    // Secret(#[from] crate::secret::Error),
    /// DHKE error
    #[error(transparent)]
    Dhke(#[from] crate::dhke::Error),
    // /// NUT10 error
    // #[error(transparent)]
    // NUT10(#[from] crate::nuts::nut10::Error),
    // /// NUT11 error
    // #[error(transparent)]
    // NUT11(#[from] crate::nuts::nut11::Error),
    /// Overflow
    #[error("Overflow")]
    Overflow,
}
