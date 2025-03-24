use nuts::{nut04, nut05};
pub use proto::bdhke::{BlindSignature, BlindedMessage, Proof};
pub use proto::keyset_rotation::keyset_rotation_service_client::KeysetRotationServiceClient;
pub use proto::keyset_rotation::keyset_rotation_service_server::{
    KeysetRotationService, KeysetRotationServiceServer,
};
pub use proto::keyset_rotation::*;
pub use proto::node::node_client::NodeClient;
pub use proto::node::node_server::{Node, NodeServer};
pub use proto::node::*;

mod proto {
    pub mod bdhke {
        tonic::include_proto!("bdhke");
    }
    pub mod node {
        tonic::include_proto!("node");
    }
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
