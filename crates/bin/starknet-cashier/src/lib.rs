pub use proto::starknet_cashier::starknet_cashier_client::StarknetCashierClient;
pub use proto::starknet_cashier::starknet_cashier_server::{
    StarknetCashier, StarknetCashierServer,
};
pub use proto::starknet_cashier::{WithdrawRequest, WithdrawResponse};

mod proto {
    pub mod starknet_cashier {
        tonic::include_proto!("starknet_cashier");
    }
}
