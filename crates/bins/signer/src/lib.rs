mod server_errors;
pub use server_errors::Error;

pub use proto::bdhke::{BlindSignature, BlindedMessage, Proof};
pub use proto::signer::signer_client::SignerClient;
pub use proto::signer::signer_server::{Signer, SignerServer};
pub use proto::signer::*;

mod proto {
    pub mod bdhke {
        tonic::include_proto!("bdhke");
    }
    pub mod signer {
        tonic::include_proto!("signer");
    }
}
