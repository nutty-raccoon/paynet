use nuts::{nut04, nut05};
pub use proto::bdhke::{BlindSignature, BlindedMessage, Proof};
#[cfg(feature = "keyset-rotation")]
pub use proto::keyset_rotation::keyset_rotation_service_client::KeysetRotationServiceClient;
#[cfg(feature = "keyset-rotation")]
pub use proto::keyset_rotation::*;
pub use proto::node::node_client::NodeClient;
pub use proto::node::*;

mod proto {
    pub mod bdhke {
        tonic::include_proto!("bdhke");
    }
    pub mod node {
        tonic::include_proto!("node");
    }
    #[cfg(feature = "keyset-rotation")]
    pub mod keyset_rotation {
        tonic::include_proto!("keyset_rotation");
    }
}

#[derive(Debug, thiserror::Error)]
#[error("The protobuf enum value is unspecified")]
pub struct UnspecifiedEnum;

impl TryFrom<MeltState> for nut05::MeltQuoteState {
    type Error = UnspecifiedEnum;

    fn try_from(value: MeltState) -> Result<Self, UnspecifiedEnum> {
        match value {
            MeltState::MlqsUnspecified => Err(UnspecifiedEnum),
            MeltState::MlqsUnpaid => Ok(nut05::MeltQuoteState::Unpaid),
            MeltState::MlqsPending => Ok(nut05::MeltQuoteState::Pending),
            MeltState::MlqsPaid => Ok(nut05::MeltQuoteState::Paid),
        }
    }
}

impl From<nut05::MeltQuoteState> for MeltState {
    fn from(value: nut05::MeltQuoteState) -> Self {
        match value {
            nut05::MeltQuoteState::Unpaid => MeltState::MlqsUnpaid,
            nut05::MeltQuoteState::Pending => MeltState::MlqsPending,
            nut05::MeltQuoteState::Paid => MeltState::MlqsPaid,
        }
    }
}

impl TryFrom<MintQuoteState> for nut04::MintQuoteState {
    type Error = UnspecifiedEnum;

    fn try_from(value: MintQuoteState) -> Result<Self, UnspecifiedEnum> {
        match value {
            MintQuoteState::MnqsUnspecified => Err(UnspecifiedEnum),
            MintQuoteState::MnqsUnpaid => Ok(nut04::MintQuoteState::Unpaid),
            MintQuoteState::MnqsPaid => Ok(nut04::MintQuoteState::Paid),
            MintQuoteState::MnqsIssued => Ok(nut04::MintQuoteState::Issued),
        }
    }
}

impl From<nut04::MintQuoteState> for MintQuoteState {
    fn from(value: nut04::MintQuoteState) -> Self {
        match value {
            nut04::MintQuoteState::Unpaid => MintQuoteState::MnqsUnpaid,
            nut04::MintQuoteState::Paid => MintQuoteState::MnqsPaid,
            nut04::MintQuoteState::Issued => MintQuoteState::MnqsIssued,
        }
    }
}

use std::hash::{DefaultHasher, Hash, Hasher};

/// Hash MintRequest to a string
/// This is used to create a unique identifier for the request
pub fn hash_mint_request(request: &MintRequest) -> u64 {
    let mut hasher = DefaultHasher::new();

    request.method.hash(&mut hasher);
    request.quote.hash(&mut hasher);
    for output in &request.outputs {
        output.amount.hash(&mut hasher);
        output.keyset_id.hash(&mut hasher);
        output.blinded_secret.hash(&mut hasher);
    }

    hasher.finish()
}

/// Hash MeltRequest to a string
/// This is used to create a unique identifier for the request
pub fn hash_melt_request(request: &MeltRequest) -> u64 {
    let mut hasher = DefaultHasher::new();

    request.method.hash(&mut hasher);
    request.quote.hash(&mut hasher);
    for input in &request.inputs {
        input.amount.hash(&mut hasher);
        input.keyset_id.hash(&mut hasher);
        input.secret.hash(&mut hasher);
        input.unblind_signature.hash(&mut hasher);
    }

    hasher.finish()
}

pub fn hash_swap_request(request: &SwapRequest) -> u64 {
    let mut hasher = DefaultHasher::new();

    for input in &request.inputs {
        input.amount.hash(&mut hasher);
        input.keyset_id.hash(&mut hasher);
        input.secret.hash(&mut hasher);
        input.unblind_signature.hash(&mut hasher);
    }
    for output in &request.outputs {
        output.amount.hash(&mut hasher);
        output.keyset_id.hash(&mut hasher);
        output.blinded_secret.hash(&mut hasher);
    }

    hasher.finish()
}
