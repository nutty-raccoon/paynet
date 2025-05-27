use starknet::{
    accounts::{Account, AccountError, ConnectedAccount},
    core::types::Call,
};
use starknet_types_core::felt::Felt;
use tracing::error;
use tracing::{Instrument, info_span};

use crate::StarknetU256;

const PAY_INVOICE_SELECTOR: Felt =
    Felt::from_hex_unchecked("0x000d5c0f26335ab142eb700850eded4619418b0f6e98c5b92a6347b68d2f2a0c");
const APPROVE_SELECTOR: Felt =
    Felt::from_hex_unchecked("0x0219209e083275171774dab1df80982e9df2096516f06319c5c6d71ae0a8480c");

pub fn generate_payment_transaction_calls(
    token_contract_address: Felt,
    invoice_payment_contract_address: Felt,
    amount: StarknetU256,
    invoice_id: Felt,
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
            invoice_id,
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
    account: &A,
    invoice_id: Felt,
    invoice_payment_contract_address: Felt,
    token_contract_address: Felt,
    amount: StarknetU256,
    payee: Felt,
) -> Result<Felt, AccountError<A::SignError>> {
    let calls = generate_payment_transaction_calls(
        token_contract_address,
        invoice_payment_contract_address,
        amount,
        invoice_id,
        payee,
    );
    // Execute the transaction
    let tx_result = account
        .execute_v3(calls.to_vec())
        .send()
        .instrument(info_span!("send-withdraw-transaction"))
        .await
        .inspect_err(|e| error!("send payment tx failed: {:?}", e))?;

    Ok(tx_result.transaction_hash)
}
