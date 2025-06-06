use std::sync::Arc;

use primitive_types::U256;
use starknet::{
    accounts::{Account, AccountError, ConnectedAccount},
    core::types::{BlockId, BlockTag, Call},
    providers::{Provider, ProviderError},
};
use starknet_types_core::felt::Felt;
use tracing::{Instrument, info_span};
use tracing::{error, info};

use crate::StarknetU256;

const PAY_INVOICE_SELECTOR: Felt =
    Felt::from_hex_unchecked("0x000d5c0f26335ab142eb700850eded4619418b0f6e98c5b92a6347b68d2f2a0c");
const APPROVE_SELECTOR: Felt =
    Felt::from_hex_unchecked("0x0219209e083275171774dab1df80982e9df2096516f06319c5c6d71ae0a8480c");

#[derive(Debug, Clone)]
pub struct WithdrawOrder {
    quote_id_hash: Felt,
    expiry: Felt,
    amount: StarknetU256,
    asset_contract_address: Felt,
    payee: Felt,
}

impl WithdrawOrder {
    pub fn new(
        quote_id_hash: Felt,
        expiry: Felt,
        amount: StarknetU256,
        asset_contract_address: Felt,
        payee: Felt,
    ) -> Self {
        Self {
            quote_id_hash,
            expiry,
            amount,
            asset_contract_address,
            payee,
        }
    }
}

pub fn generate_payment_transaction_calls<'a>(
    invoice_payment_contract_address: Felt,
    orders: impl ExactSizeIterator<Item = &'a WithdrawOrder> + Clone,
) -> Vec<Call> {
    let mut amounts_to_approve: Vec<(Felt, primitive_types::U256)> = vec![];

    let n_orders = orders.len();
    // First push the aggregated approve calls
    for order in orders.clone() {
        let amount = U256::from(&order.amount);
        match amounts_to_approve
            .iter_mut()
            .find(|c| c.0 == order.asset_contract_address)
        {
            Some((_, a)) => *a = a.checked_add(amount).unwrap(),
            None => amounts_to_approve.push((order.asset_contract_address, amount)),
        }
    }

    let mut calls = Vec::with_capacity(amounts_to_approve.len() + n_orders);
    for amount_to_approve in amounts_to_approve {
        let amount = StarknetU256::from(amount_to_approve.1);
        calls.push(Call {
            to: amount_to_approve.0,
            selector: APPROVE_SELECTOR,
            calldata: vec![invoice_payment_contract_address, amount.low, amount.high],
        });
    }
    for order in orders {
        calls.push(Call {
            to: invoice_payment_contract_address,
            selector: PAY_INVOICE_SELECTOR,
            calldata: vec![
                order.quote_id_hash,
                order.expiry,
                order.asset_contract_address,
                order.amount.low,
                order.amount.high,
                order.payee,
            ],
        });
    }

    calls
}

pub fn generate_single_payment_transaction_calls(
    invoice_payment_contract_address: Felt,
    quote_id_hash: Felt,
    expiry: Felt,
    token_contract_address: Felt,
    amount: StarknetU256,
    payee: Felt,
) -> [Call; 2] {
    // First approve our invoice contract to spend the account funds
    let approve_call = Call {
        to: token_contract_address,
        selector: APPROVE_SELECTOR,
        calldata: vec![invoice_payment_contract_address, amount.low, amount.high],
    };
    // Then do the actual transfer through our invoice contract
    let transfer_call = Call {
        to: invoice_payment_contract_address,
        selector: PAY_INVOICE_SELECTOR,
        calldata: vec![
            quote_id_hash,
            expiry,
            token_contract_address,
            amount.low,
            amount.high,
            payee,
        ],
    };

    [approve_call, transfer_call]
}

pub async fn sign_and_send_payment_transactions<
    A: Account + ConnectedAccount + Sync + std::fmt::Debug,
>(
    account: Arc<A>,
    invoice_payment_contract_address: Felt,
    withdrawal_orders: impl ExactSizeIterator<Item = &WithdrawOrder> + Clone,
) -> Result<Felt, Error<A>> {
    let calls =
        generate_payment_transaction_calls(invoice_payment_contract_address, withdrawal_orders);

    let calls_debug_string = format!("{:?}", calls);
    let nonce = account
        .provider()
        .get_nonce(BlockId::Tag(BlockTag::Pending), account.address())
        .await?;
    // Execute the transaction
    let tx_result = account
        .execute_v3(calls)
        .nonce(nonce)
        .send()
        .instrument(info_span!("send-withdraw-transaction"))
        .await
        .inspect(|tx_result|
            info!(name: "send-payment-transaction", name = "send-payment-transaction", calls = calls_debug_string, ?tx_result)
        )
        .inspect_err(|error|
            error!(name: "send-payment-transaction", name = "send-payment-transaction", calls = calls_debug_string, %error),
        )?;

    Ok(tx_result.transaction_hash)
}

pub async fn sign_and_send_single_payment_transactions<
    A: Account + ConnectedAccount + Sync + std::fmt::Debug,
>(
    account: Arc<A>,
    invoice_payment_contract_address: Felt,
    withdrawal_order: WithdrawOrder,
) -> Result<(Felt, Felt), Error<A>> {
    let calls = generate_single_payment_transaction_calls(
        invoice_payment_contract_address,
        withdrawal_order.quote_id_hash,
        withdrawal_order.expiry,
        withdrawal_order.asset_contract_address,
        withdrawal_order.amount,
        withdrawal_order.payee,
    );

    let calls_debug_string = format!("{:?}", calls);
    let nonce = account.get_nonce().await?;
    // Execute the transaction
    let tx_result = account
        .execute_v3(calls.to_vec())
        .nonce(nonce)
        .send()
        .instrument(info_span!("send-withdraw-transaction"))
        .await
        .inspect(|tx_result|
            info!(name: "send-payment-transaction", name = "send-payment-transaction", calls = calls_debug_string, ?tx_result)
        )
        .inspect_err(|error|
            error!(name: "send-payment-transaction", name = "send-payment-transaction", calls = calls_debug_string, %error),
        )?;

    Ok((tx_result.transaction_hash, nonce))
}

#[derive(Debug, thiserror::Error)]
pub enum Error<A: Account> {
    #[error(transparent)]
    Account(#[from] AccountError<A::SignError>),
    #[error(transparent)]
    Provider(#[from] ProviderError),
}
