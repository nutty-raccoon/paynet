use nuts::{nut03::SwapRequest as NutSwapRequest, nut04, nut05};
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

impl TryFrom<MeltQuoteState> for nut05::MeltQuoteState {
    type Error = UnspecifiedEnum;

    fn try_from(value: MeltQuoteState) -> Result<Self, UnspecifiedEnum> {
        match value {
            MeltQuoteState::MlqsUnspecified => Err(UnspecifiedEnum),
            MeltQuoteState::MlqsUnpaid => Ok(nut05::MeltQuoteState::Unpaid),
            MeltQuoteState::MlqsPending => Ok(nut05::MeltQuoteState::Pending),
            MeltQuoteState::MlqsPaid => Ok(nut05::MeltQuoteState::Paid),
        }
    }
}

impl From<nut05::MeltQuoteState> for MeltQuoteState {
    fn from(value: nut05::MeltQuoteState) -> Self {
        match value {
            nut05::MeltQuoteState::Unpaid => MeltQuoteState::MlqsUnpaid,
            nut05::MeltQuoteState::Pending => MeltQuoteState::MlqsPending,
            nut05::MeltQuoteState::Paid => MeltQuoteState::MlqsPaid,
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
