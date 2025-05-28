use starknet::accounts::{Account, ExecutionEncoding, SingleOwnerAccount};
use starknet::core::types::Felt;
use starknet::providers::Provider;
use starknet::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet::signers::{LocalWallet, SigningKey};
use starknet_cashier::{ConfigRequest, ConfigResponse, WithdrawRequest, WithdrawResponse};
use starknet_types::transactions::sign_and_send_payment_transactions;
use starknet_types::{Asset, StarknetU256, felt_to_short_string, is_valid_starknet_address};
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::env_vars::read_env_variables;
use starknet_types::constants::ON_CHAIN_CONSTANTS;

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
            ExecutionEncoding::New,
        );

        Ok(Self {
            account: Arc::new(account),
        })
    }
}

#[tonic::async_trait]
impl starknet_cashier::StarknetCashier for StarknetCashierState {
    async fn config(
        &self,
        _withdraw_request: Request<ConfigRequest>,
    ) -> Result<Response<ConfigResponse>, Status> {
        let chain_id = self.account.chain_id();
        let chain_id = felt_to_short_string(chain_id);

        Ok(Response::new(ConfigResponse { chain_id }))
    }

    async fn withdraw(
        &self,
        withdraw_request: Request<WithdrawRequest>,
    ) -> Result<Response<WithdrawResponse>, Status> {
        let request = withdraw_request.into_inner();

        let invoice_id = Felt::from_bytes_be_slice(&request.invoice_id);
        let amount = StarknetU256::from_bytes_slice(&request.amount)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let asset =
            Asset::from_str(&request.asset).map_err(|e| Status::invalid_argument(e.to_string()))?;

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
        let on_chain_constants = ON_CHAIN_CONSTANTS
            .get(&chain_id)
            .ok_or_else(|| Status::internal("invalid chain id"))?;
        let asset_contract_address = on_chain_constants
            .assets_contract_address
            .get_contract_address_for_asset(asset)
            .ok_or_else(|| Status::invalid_argument("bad assset"))?;

        let payee_address = Felt::from_bytes_be_slice(&request.payee);
        if !is_valid_starknet_address(&payee_address) {
            return Err(Status::invalid_argument(format!(
                "invalid payee address: {}",
                payee_address
            )));
        }

        match sign_and_send_payment_transactions(
            &self.account,
            invoice_id,
            on_chain_constants.invoice_payment_contract_address,
            asset_contract_address,
            amount,
            payee_address,
            request.expiry,
        )
        .await
        {
            Ok(tx_hash) => Ok(Response::new(WithdrawResponse {
                tx_hash: tx_hash.to_bytes_be().to_vec(),
            })),
            Err(err) => Err(Status::internal(format!(
                "Failed to execute transaction: {}",
                err
            ))),
        }
    }
}
