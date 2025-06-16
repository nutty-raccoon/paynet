use anyhow::anyhow;
use log::error;
use starknet::{
    accounts::{Account, ConnectedAccount, ExecutionEncoding, SingleOwnerAccount},
    providers::{JsonRpcClient, jsonrpc::HttpTransport},
    signers::{LocalWallet, SigningKey},
};
use starknet_types_core::felt::Felt;
use url::Url;

use crate::env_variables::EnvVariables;
use crate::errors::{Error, Result};

pub fn init_account(
    env: EnvVariables,
) -> Result<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>> {
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        Felt::from_hex(&env.private_key).map_err(|e| Error::Other(e.into()))?,
    ));
    let address = Felt::from_hex(&env.account_address).map_err(|e| Error::Other(e.into()))?;

    let provider = JsonRpcClient::new(HttpTransport::new(
        Url::parse(&env.rpc_url).map_err(|e| Error::Other(e.into()))?,
    ));

    let account = SingleOwnerAccount::new(
        provider,
        signer,
        address,
        Felt::from_bytes_be_slice("SN_DEVNET".as_bytes()),
        ExecutionEncoding::New,
    );

    Ok(account)
}

pub async fn pay_invoice(invoice_json: String, env: EnvVariables) -> Result<()> {
    let account = init_account(env)?;

    let calls: [starknet_types::Call; 2] = serde_json::from_str(&invoice_json)?;
    let tx_hash = account
        .execute_v3(calls.into_iter().map(Into::into).collect())
        .send()
        .await
        .inspect_err(|e| error!("send payment tx failed: {:?}", e))
        .map_err(|e| Error::Other(e.into()))?
        .transaction_hash;

    watch_tx(account.provider(), tx_hash).await?;

    Ok(())
}

pub async fn watch_tx<P>(provider: P, transaction_hash: Felt) -> Result<()>
where
    P: starknet::providers::Provider,
{
    loop {
        use starknet::core::types::{StarknetError, TransactionExecutionStatus, TransactionStatus};
        use starknet::providers::ProviderError;
        match provider.get_transaction_status(transaction_hash).await {
            Ok(TransactionStatus::AcceptedOnL2(TransactionExecutionStatus::Succeeded)) => {
                return Ok(());
            }
            Ok(TransactionStatus::AcceptedOnL2(TransactionExecutionStatus::Reverted)) => {
                return Err(Error::Other(anyhow!("tx reverted")));
            }
            Ok(TransactionStatus::Received) => {}
            Ok(TransactionStatus::Rejected) => return Err(Error::Other(anyhow!("tx rejected"))),
            Err(ProviderError::StarknetError(StarknetError::TransactionHashNotFound)) => {}
            Err(err) => return Err(err.into()),
            Ok(TransactionStatus::AcceptedOnL1(_)) => unreachable!(),
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
