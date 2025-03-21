use starknet::accounts::{Account, ExecutionEncoding, SingleOwnerAccount};
use starknet::core::types::{Call, Felt};
use starknet::providers::Provider;
use starknet::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet::signers::{LocalWallet, SigningKey};
use starknet_cashier::{WithdrawRequest, WithdrawResponse};
use starknet_types::Asset;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::chain_constants::ONCHAIN_CONSTANTS;
use crate::env_vars::read_env_variables;

const PAY_INVOICE_SELECTOR: Felt =
    Felt::from_hex_unchecked("0x000d5c0f26335ab142eb700850eded4619418b0f6e98c5b92a6347b68d2f2a0c");
const APPROVE_SELECTOR: Felt =
    Felt::from_hex_unchecked("0x0219209e083275171774dab1df80982e9df2096516f06319c5c6d71ae0a8480c");

#[derive(Debug, Clone)]
pub struct StarknetCashierState {
    account: Arc<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>>,
}

impl StarknetCashierState {
    pub async fn new() -> anyhow::Result<Self> {
        // Get environment variables
        let (rpc_url, private_key, address, _) = read_env_variables()?;

        // Create provider
        let provider = JsonRpcClient::new(HttpTransport::new(rpc_url));

        // Create signer
        let signer = LocalWallet::from(SigningKey::from_secret_scalar(private_key));

        // Create account
        let chain_id = provider.chain_id().await?;
        let account = SingleOwnerAccount::new(
            provider.clone(),
            signer,
            address,
            chain_id,
            ExecutionEncoding::Legacy,
        );

        Ok(Self {
            account: Arc::new(account),
        })
    }

    pub async fn sign_and_send_erc20_transfer(
        &self,
        invoice_payment_contract_address: Felt,
        token_contract_address: Felt,
        recipient: Felt,
        amount: Felt,
    ) -> anyhow::Result<Felt> {
        // First approve our invoice contract to spend the account funds
        let approve_call = Call {
            to: token_contract_address,
            selector: APPROVE_SELECTOR,
            calldata: vec![invoice_payment_contract_address, amount],
        };
        // Then do the actual transfer through our invoice contract
        let transfer_call = Call {
            to: invoice_payment_contract_address,
            selector: PAY_INVOICE_SELECTOR,
            calldata: vec![recipient, amount],
        };

        // Execute the transaction
        let tx_result = self
            .account
            .execute_v3(vec![approve_call, transfer_call])
            .send()
            .await?;

        Ok(tx_result.transaction_hash)
    }
}

#[tonic::async_trait]
impl starknet_cashier::StarknetCashier for StarknetCashierState {
    async fn withdraw(
        &self,
        withdraw_request: Request<WithdrawRequest>,
    ) -> Result<Response<WithdrawResponse>, Status> {
        let request = withdraw_request.into_inner();

        let chain_id = self.account.chain_id();
        // Safe because Felt short string don't contain non-utf8 characters
        let chain_id = unsafe {
            String::from_utf8_unchecked(
                chain_id
                    .to_bytes_be()
                    .into_iter()
                    .skip_while(|&b| b == 0)
                    .collect(),
            )
        };

        let on_chain_constants = ONCHAIN_CONSTANTS
            .get(&chain_id)
            .ok_or_else(|| Status::internal("invalid chain id"))?;

        let asset =
            Asset::from_str(&request.asset).map_err(|e| Status::invalid_argument(e.to_string()))?;
        let asset_contract_address = on_chain_constants
            .assets_contract_address
            .get(asset.as_ref())
            .ok_or_else(|| Status::invalid_argument("bad assset"))?;
        let amount = Felt::from_bytes_be_slice(&request.amount);
        let payee_address = Felt::from_bytes_be_slice(&request.payee);

        match self
            .sign_and_send_erc20_transfer(
                on_chain_constants.invoice_payment_contract_address,
                *asset_contract_address,
                payee_address,
                amount,
            )
            .await
        {
            Ok(tx_hash) => Ok(Response::new(WithdrawResponse {
                tx_hash: tx_hash
                    .to_bytes_be()
                    .into_iter()
                    .skip_while(|&b| b == 0)
                    .collect(),
            })),
            Err(err) => Err(Status::internal(format!(
                "Failed to execute transaction: {}",
                err
            ))),
        }
    }
}
