#[derive(Debug, Clone)]
pub struct EnvVariables {
    pub node_url: String,
    pub rpc_url: String,
    pub private_key: String,
    pub account_address: String,
    pub chain_id: String,
}

#[cfg(feature = "strk")]
pub mod starknet {
    use anyhow::{Result, anyhow};
    use log::error;
    use starknet::{
        accounts::{Account, ConnectedAccount, ExecutionEncoding, SingleOwnerAccount},
        core::{
            types::{Call, ExecutionResult, StarknetError, TransactionStatus},
            utils::cairo_short_string_to_felt,
        },
        providers::{JsonRpcClient, ProviderError, jsonrpc::HttpTransport},
        signers::{LocalWallet, SigningKey},
    };
    use starknet_types_core::felt::Felt;
    use url::Url;

    use super::EnvVariables;

    pub fn init_account(
        env: EnvVariables,
    ) -> Result<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>> {
        let signer = LocalWallet::from(SigningKey::from_secret_scalar(Felt::from_hex(
            &env.private_key,
        )?));
        let address = Felt::from_hex(&env.account_address)?;

        let provider = JsonRpcClient::new(HttpTransport::new(Url::parse(&env.rpc_url)?));

        let account = SingleOwnerAccount::new(
            provider,
            signer,
            address,
            cairo_short_string_to_felt(env.chain_id.as_str()).unwrap(),
            ExecutionEncoding::New,
        );

        Ok(account)
    }

    pub async fn pay_invoices(calls: Vec<Call>, env: EnvVariables) -> Result<()> {
        let account = init_account(env)?;

        let tx_hash = account
            .execute_v3(calls)
            .send()
            .await
            .inspect_err(|e| error!("send payment tx failed: {:?}", e))?
            .transaction_hash;

        watch_tx(account.provider(), tx_hash).await?;

        Ok(())
    }

    pub async fn watch_tx<P>(provider: P, tx_hash: Felt) -> Result<()>
    where
        P: starknet::providers::Provider,
    {
        loop {
            match provider.get_transaction_status(tx_hash).await {
                Ok(TransactionStatus::AcceptedOnL2(ExecutionResult::Succeeded)) => {
                    break;
                }
                Ok(TransactionStatus::AcceptedOnL2(ExecutionResult::Reverted { reason })) => {
                    return Err(anyhow!("tx reverted: {}", reason));
                }
                Ok(TransactionStatus::Received) => {}
                Ok(TransactionStatus::Rejected) => {
                    return Err(anyhow!("tx rejected"));
                }
                Err(ProviderError::StarknetError(StarknetError::TransactionHashNotFound)) => {}
                Err(err) => return Err(err.into()),
                Ok(TransactionStatus::AcceptedOnL1(_)) => unreachable!(),
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        // Wait for block to not be pending anymore
        loop {
            if let starknet::core::types::ReceiptBlock::Block {
                block_hash: _,
                block_number: _,
            } = provider.get_transaction_receipt(tx_hash).await?.block
            {
                break;
            } else {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }
        }

        Ok(())
    }
}
