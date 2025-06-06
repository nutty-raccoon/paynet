use nuts::nut05::MeltQuoteState;
use serde::{Deserialize, Serialize};
use starknet_types::{Asset, STARKNET_STR, StarknetU256, Unit, constants::ON_CHAIN_CONSTANTS};

use liquidity_source::{WithdrawAmount, WithdrawInterface, WithdrawRequest};
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
    WithdrawOrder, sign_and_send_payment_transactions, sign_and_send_single_payment_transactions,
};
use tokio::{sync::mpsc, time::sleep};
use tracing::{error, info};

use crate::StarknetInvoiceId;

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
    SendWithdrawOrder(#[from] SendError<WithdrawOrder>),
    #[error("asset {0} not found in on-chain constants")]
    AssetNotFound(Asset),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeltPaymentRequest {
    pub payee: Felt,
    pub asset: Asset,
}

impl WithdrawRequest for MeltPaymentRequest {
    fn asset(&self) -> Asset {
        self.asset
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct StarknetU256WithdrawAmount(pub StarknetU256);

impl WithdrawAmount for StarknetU256WithdrawAmount {
    fn convert_from(unit: Unit, amount: nuts::Amount) -> Self {
        Self(StarknetU256::from(unit.convert_amount_into_u256(amount)))
    }
}

#[derive(Debug, Clone)]
pub struct Withdrawer {
    withdraw_order_sender: mpsc::UnboundedSender<WithdrawOrder>,
}

impl Withdrawer {
    pub fn new(account: Arc<OurAccount>, invoice_payment_contract_address: Felt) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let _join_handle = tokio::spawn(process_withdraw_requests(
            account,
            rx,
            invoice_payment_contract_address,
        ));

        Self {
            withdraw_order_sender: tx,
        }
    }
}

#[async_trait::async_trait]
impl WithdrawInterface for Withdrawer {
    type Error = Error;
    type Request = MeltPaymentRequest;
    type Amount = StarknetU256WithdrawAmount;
    type InvoiceId = StarknetInvoiceId;

    fn deserialize_payment_request(&self, raw_json_string: &str) -> Result<Self::Request, Error> {
        let pr = serde_json::from_str::<Self::Request>(raw_json_string)
            .map_err(Error::InvalidPaymentRequest)?;

        if !is_valid_starknet_address(&pr.payee) {
            return Err(Error::InvalidStarknetAddress(pr.payee));
        }
        Ok(pr)
    }

    async fn proceed_to_payment(
        &mut self,
        quote_id: Uuid,
        melt_payment_request: MeltPaymentRequest,
        amount: Self::Amount,
        expiry: u64,
    ) -> Result<MeltQuoteState, Error> {
        let quote_id_hash =
            Felt::from_bytes_be(bitcoin_hashes::Sha256::hash(quote_id.as_bytes()).as_byte_array());

        let on_chain_constants = ON_CHAIN_CONSTANTS.get(STARKNET_STR).unwrap();
        let asset_contract_address = on_chain_constants
            .assets_contract_address
            .get_contract_address_for_asset(melt_payment_request.asset)
            .ok_or(Error::AssetNotFound(asset))?;

        let payee_address = Felt::from_bytes_be_slice(&request.payee);
        if !is_valid_starknet_address(&payee_address) {
            return Err(Error::InvalidStarknetAddress(payee_address));
        }

        self.withdraw_order_sender.send(WithdrawOrder::new(
            quote_id_hash,
            expiry.into(),
            amount.0,
            asset_contract_address,
            payee_address,
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

    Ok(())
}

pub async fn process_withdraw_requests(
    account: Arc<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>>,
    mut withdraw_queue: mpsc::UnboundedReceiver<WithdrawOrder>,
    invoice_payment_contract_address: Felt,
) -> Result<(), Error> {
    let mut orders = Vec::new();

    loop {
        let withdraw_order = withdraw_queue.recv().await.ok_or(Error::ChannelClosed)?;

        let (tx_hash, tx_nonce) = sign_and_send_single_payment_transactions(
            account.clone(),
            invoice_payment_contract_address,
            withdraw_order,
        )
        .await?;

        let mut tx_handle = tokio::spawn(wait_for_tx_completion(account.clone(), tx_hash));

        loop {
            if !tx_handle.is_finished() {
                withdraw_queue.recv_many(&mut orders, 10).await;
            } else if !orders.is_empty() {
                let orders = std::mem::take(&mut orders);
                let tx_hash = sign_and_send_payment_transactions(
                    account.clone(),
                    invoice_payment_contract_address,
                    orders.iter(),
                )
                .await?;
                tx_handle = tokio::spawn(wait_for_tx_completion(account.clone(), tx_hash));
            } else {
                break;
            }
        }
    }
}
