#[cfg(feature = "mock")]
mod mock;

#[cfg(feature = "mock")]
pub use mock::*;
#[cfg(not(feature = "mock"))]
pub use not_mock::*;

use serde::{Deserialize, Serialize};
use ethereum_types::{Asset, EthereumAddress, EthereumU256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeltPaymentRequest {
    pub payee: EthereumAddress,
    pub asset: Asset,
    pub amount: EthereumU256,
}

#[cfg(not(feature = "mock"))]
mod not_mock {
    use std::{sync::Arc, time::Duration};

    use async_trait::async_trait;
    use ethereum_types::{
        Asset, ChainId, EthereumAddress, Unit, constants::ON_CHAIN_CONSTANTS, is_valid_ethereum_address,
    };
    use liquidity_source::WithdrawInterface;
    use nuts::{Amount, nut05::MeltQuoteState};
    use primitive_types::{H256, U256};
    use tokio::sync::mpsc;
    use uuid::Uuid;

    use crate::EthereumInvoiceId;
    use super::MeltPaymentRequest;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("invalid payment request: {0}")]
        InvalidPaymentRequest(#[from] serde_json::Error),
        #[error("invalid ethereum address: {0:?}")]
        InvalidEthereumAddress(EthereumAddress),
        #[error("asset {0} not found in on-chain constants")]
        AssetNotFound(Asset),
        #[error("failed to send withdraw order: {0}")]
        WithdrawOrderSend(#[from] mpsc::error::SendError<WithdrawOrder>),
    }

    #[derive(Debug, Clone)]
    pub struct Withdrawer {
        chain_id: ChainId,
        withdraw_order_sender: mpsc::UnboundedSender<WithdrawOrder>,
    }

    impl Withdrawer {
        pub fn new(
            chain_id: ChainId,
            withdraw_order_sender: mpsc::UnboundedSender<WithdrawOrder>,
        ) -> Self {
            Self {
                chain_id,
                withdraw_order_sender,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct WithdrawOrder {
        pub quote_id_hash: H256,
        pub expiry: U256,
        pub amount: U256,
        pub token_contract_address: Option<EthereumAddress>,
        pub payee: EthereumAddress,
    }

    impl WithdrawOrder {
        pub fn new(
            quote_id_hash: H256,
            expiry: U256,
            amount: U256,
            token_contract_address: Option<EthereumAddress>,
            payee: EthereumAddress,
        ) -> Self {
            Self {
                quote_id_hash,
                expiry,
                amount,
                token_contract_address,
                payee,
            }
        }
    }

    #[async_trait::async_trait]
    impl WithdrawInterface for Withdrawer {
        type Error = Error;
        type Request = MeltPaymentRequest;
        type Unit = Unit;
        type InvoiceId = EthereumInvoiceId;

        fn compute_total_amount_expected(
            &self,
            request: Self::Request,
            unit: Self::Unit,
            fee: Amount,
        ) -> Result<Amount, Self::Error> {
            let fee_u256 = unit.convert_amount_into_u256(fee);
            let total_amount = request.amount + fee_u256;
            
            unit.convert_u256_into_amount(total_amount)
                .map_err(|_| Error::InvalidPaymentRequest(
                    serde_json::Error::custom("Amount conversion failed")
                ))
        }

        fn deserialize_payment_request(
            &self,
            raw_json_string: &str,
        ) -> Result<Self::Request, Error> {
            let pr = serde_json::from_str::<Self::Request>(raw_json_string)
                .map_err(Error::InvalidPaymentRequest)?;

            if !is_valid_ethereum_address(&format!("{:?}", pr.payee)) {
                return Err(Error::InvalidEthereumAddress(pr.payee));
            }

            Ok(pr)
        }

        async fn proceed_to_payment(
            &mut self,
            quote_id: Uuid,
            melt_payment_request: MeltPaymentRequest,
            expiry: u64,
        ) -> Result<MeltQuoteState, Error> {
            let quote_id_hash = H256::from_slice(
                bitcoin_hashes::sha256::Hash::hash(quote_id.as_bytes()).as_byte_array()
            );

            let on_chain_constants = ON_CHAIN_CONSTANTS.get(self.chain_id.as_str()).unwrap();
            let token_contract_address = match melt_payment_request.asset {
                Asset::Eth => None, // ETH is native
                _ => on_chain_constants
                    .assets_contract_address
                    .get_contract_address_for_asset(melt_payment_request.asset),
            };

            if melt_payment_request.asset != Asset::Eth && token_contract_address.is_none() {
                return Err(Error::AssetNotFound(melt_payment_request.asset));
            }

            self.withdraw_order_sender.send(WithdrawOrder::new(
                quote_id_hash,
                expiry.into(),
                melt_payment_request.amount,
                token_contract_address,
                melt_payment_request.payee,
            ))?;

            Ok(MeltQuoteState::Pending)
        }
    }

    // Placeholder for Ethereum transaction processing
    // This would be implemented similar to the Starknet version but using ethers-rs
    pub async fn process_withdraw_requests(
        // ethereum_client: Arc<EthereumClient>, // Would use ethers-rs client
        mut withdraw_queue: mpsc::UnboundedReceiver<WithdrawOrder>,
        invoice_payment_contract_address: EthereumAddress,
    ) -> Result<(), Error> {
        let mut orders = Vec::new();
        
        // TODO: Implement Ethereum transaction processing
        // This would involve:
        // 1. Batching withdraw orders
        // 2. Creating Ethereum transactions
        // 3. Signing and sending transactions
        // 4. Waiting for confirmations
        
        loop {
            withdraw_queue.recv_many(&mut orders, 10).await;
            
            if !orders.is_empty() {
                tracing::info!("Processing {} Ethereum withdraw orders", orders.len());
                
                // Process orders (placeholder)
                for order in &orders {
                    tracing::debug!("Processing withdraw order: {:?}", order);
                }
                
                orders.clear();
            }
        }
    }
}
