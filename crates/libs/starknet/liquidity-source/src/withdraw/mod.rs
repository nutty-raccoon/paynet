#[cfg(feature = "mock")]
mod mock;

#[cfg(feature = "mock")]
pub use mock::*;
#[cfg(not(feature = "mock"))]
pub use not_mock::*;

use serde::{Deserialize, Serialize};
use starknet_types::{Asset, StarknetU256};
use starknet_types_core::felt::Felt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeltPaymentRequest {
    pub payee: Felt,
    pub asset: Asset,
    pub amount: StarknetU256,
}

#[cfg(not(feature = "mock"))]
mod not_mock {
    use num_traits::CheckedAdd;
    use nuts::{Amount, nut05::MeltQuoteState};
    use starknet_types::{
        Asset, AssetToUnitConversionError, ChainId, Unit, constants::ON_CHAIN_CONSTANTS,
    };

    use liquidity_source::WithdrawInterface;
    use starknet_types::is_valid_starknet_address;
    use uuid::Uuid;

    use std::{sync::Arc, time::Duration};

    use starknet::{
        accounts::{Account, ConnectedAccount, SingleOwnerAccount},
        core::types::{Felt, TransactionExecutionStatus, TransactionStatus},
        providers::{JsonRpcClient, Provider, ProviderError, jsonrpc::HttpTransport},
        signers::LocalWallet,
    };
    use starknet_types::transactions::{
        WithdrawOrder, sign_and_send_payment_transactions,
        sign_and_send_single_payment_transactions,
    };
    use tokio::{sync::mpsc, time::sleep};
    use tracing::{error, info};

    use crate::StarknetInvoiceId;

    use super::MeltPaymentRequest;

    type OurAccount = SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("invalid payment request json string: {0}")]
        InvalidPaymentRequest(#[from] serde_json::Error),
        #[error("invalid starknet address: {0}")]
        InvalidStarknetAddress(Felt),
        #[error("failed to send transaction: {0}")]
        Transaction(#[from] starknet_types::transactions::Error<OurAccount>),
        #[error("withdraw order channel has been closed")]
        ChannelClosed,
        #[error("failed to emit confirmation for tx {0}")]
        TransactionConfirmation(Felt),
        #[error("failed to get transaction status from node: {0}")]
        GetTransactionStatus(ProviderError),
        #[error("failed to get nonce from node: {0}")]
        GetNonce(ProviderError),
        #[error("failed to send withdraw order through channel: {0}")]
        SendWithdrawOrder(#[from] mpsc::error::SendError<WithdrawOrder>),
        #[error("asset {0} not found in on-chain constants")]
        AssetNotFound(Asset),
        #[error("failed to acquire a conneciton from the pool: {0}")]
        PgPool(sqlx::Error),
        #[error("failed to register transaction hash in melt_quote table: {0}")]
        RegisterTxHash(sqlx::Error),
        #[error("failed to convert request values to nodes values: {0}")]
        Conversion(#[from] AssetToUnitConversionError),
        #[error("amount overflow")]
        Overflow,
        #[error("unsupported asset `{0}` for unit `{1}`")]
        InvalidAssetForUnit(Asset, Unit),
    }

    #[derive(Debug, Clone)]
    pub struct Withdrawer {
        chain_id: ChainId,
        withdraw_order_sender: mpsc::UnboundedSender<WithdrawOrder>,
    }

    impl Withdrawer {
        pub fn new(
            chain_id: ChainId,
            account: Arc<OurAccount>,
            invoice_payment_contract_address: Felt,
        ) -> Self {
            let (tx, rx) = mpsc::unbounded_channel();

            let _join_handle = tokio::spawn(async move {
                let res =
                    process_withdraw_requests(account, rx, invoice_payment_contract_address).await;

                match res {
                    Ok(_) => error!(name: "cashier-worker", error = "returned"),
                    Err(err) => error!(name: "cashier-worker", error = %err),
                }
            });

            Self {
                chain_id,
                withdraw_order_sender: tx,
            }
        }
    }

    #[async_trait::async_trait]
    impl WithdrawInterface for Withdrawer {
        type Error = Error;
        type Request = MeltPaymentRequest;
        type Unit = Unit;
        type InvoiceId = StarknetInvoiceId;

        fn deserialize_payment_request(
            &self,
            raw_json_string: &str,
        ) -> Result<Self::Request, Error> {
            let pr = serde_json::from_str::<Self::Request>(raw_json_string)
                .map_err(Error::InvalidPaymentRequest)?;

            if !is_valid_starknet_address(&pr.payee) {
                return Err(Error::InvalidStarknetAddress(pr.payee));
            }

            Ok(pr)
        }

        fn compute_total_amount_expected(
            &self,
            request: Self::Request,
            unit: Unit,
            fee: Amount,
        ) -> Result<nuts::Amount, Self::Error> {
            if !unit.is_asset_supported(request.asset) {
                return Err(Error::InvalidAssetForUnit(request.asset, unit));
            }

            let (amount, rem) = request
                .asset
                .convert_to_amount_of_unit(request.amount.clone().into(), unit)?;

            if fee == Amount::ZERO {
                if rem.is_zero() {
                    Ok(amount)
                } else {
                    amount.checked_add(&Amount::ONE).ok_or(Error::Overflow)
                }
            } else {
                amount.checked_add(&fee).ok_or(Error::Overflow)
            }
        }

        async fn proceed_to_payment(
            &mut self,
            quote_id: Uuid,
            melt_payment_request: MeltPaymentRequest,
            expiry: u64,
        ) -> Result<MeltQuoteState, Error> {
            let quote_id_hash = Felt::from_bytes_be(
                bitcoin_hashes::Sha256::hash(quote_id.as_bytes()).as_byte_array(),
            );

            let on_chain_constants = ON_CHAIN_CONSTANTS.get(self.chain_id.as_str()).unwrap();
            let asset_contract_address = on_chain_constants
                .assets_contract_address
                .get_contract_address_for_asset(melt_payment_request.asset)
                .ok_or(Error::AssetNotFound(melt_payment_request.asset))?;

            self.withdraw_order_sender.send(WithdrawOrder::new(
                quote_id_hash,
                expiry.into(),
                melt_payment_request.amount,
                asset_contract_address,
                melt_payment_request.payee,
            ))?;

            Ok(MeltQuoteState::Pending)
        }
    }

    async fn wait_for_tx_completion<A: Account + ConnectedAccount + Sync>(
        account: Arc<A>,
        tx_hash: Felt,
    ) -> Result<(), Error> {
        loop {
            match account
                .provider()
                .get_transaction_status(tx_hash)
                .await
                .map_err(Error::GetTransactionStatus)?
            {
                TransactionStatus::Received => {
                    sleep(Duration::from_millis(500)).await;
                    continue;
                }
                TransactionStatus::AcceptedOnL2(TransactionExecutionStatus::Succeeded) => {
                    info!(name: "withdraw-tx-result", name =  "withdraw-tx-result", tx_hash = tx_hash.to_hex_string(), status = "succeeded");
                    break;
                }
                TransactionStatus::AcceptedOnL2(TransactionExecutionStatus::Reverted) => {
                    error!(name: "withdraw-tx-result", name =  "withdraw-tx-result", tx_hash = tx_hash.to_hex_string(), status = "reverted");
                    break;
                }
                TransactionStatus::Rejected => {
                    error!(name: "withdraw-tx-result", name = "withdraw-tx-result", tx_hash = tx_hash.to_hex_string(), status = "rejected");
                    break;
                }
                TransactionStatus::AcceptedOnL1(_) => unreachable!(),
            }
        }
        loop {
            if let starknet::core::types::ReceiptBlock::Block {
                block_hash: _,
                block_number: _,
            } = account
                .provider()
                .get_transaction_receipt(tx_hash)
                .await
                .map_err(Error::GetTransactionStatus)?
                .block
            {
                break;
            } else {
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        }

        Ok(())
    }

    pub async fn process_withdraw_requests(
        account: Arc<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>>,
        mut withdraw_queue: mpsc::UnboundedReceiver<WithdrawOrder>,
        invoice_payment_contract_address: Felt,
    ) -> Result<(), Error> {
        let mut orders = Vec::new();
        let mut tx_handle: Option<tokio::task::JoinHandle<Result<(), Error>>> = None;

        // TODO: retry logic
        loop {
            match tx_handle.as_ref() {
                None if orders.is_empty() => {
                    withdraw_queue.recv_many(&mut orders, 10).await;
                }
                Some(txh) => {
                    while !txh.is_finished() {
                        let _ = tokio::time::timeout(
                            Duration::from_secs(1),
                            withdraw_queue.recv_many(&mut orders, 10),
                        )
                        .await;
                    }

                    tx_handle = None;
                }
                _ => {
                    let tx_hash = if orders.len() == 1 {
                        sign_and_send_single_payment_transactions(
                            account.clone(),
                            invoice_payment_contract_address,
                            &orders[0],
                        )
                        .await?
                    } else {
                        sign_and_send_payment_transactions(
                            account.clone(),
                            invoice_payment_contract_address,
                            orders.iter(),
                        )
                        .await?
                    };

                    orders.clear();

                    tx_handle = Some(tokio::spawn(wait_for_tx_completion(
                        account.clone(),
                        tx_hash,
                    )));
                }
            }
        }
    }
}
