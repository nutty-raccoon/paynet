use std::{path::PathBuf, sync::Arc, time::Duration};

use anyhow::{Error, anyhow};
use clap::{Parser, ValueHint};
use log::{debug, info};
use starknet::{
    accounts::{Account, ConnectedAccount, ExecutionEncoding, SingleOwnerAccount},
    contract::ContractFactory,
    core::types::{
        BlockId, BlockTag, Felt, StarknetError, TransactionExecutionStatus, TransactionStatus,
        contract::SierraClass,
    },
    providers::{JsonRpcClient, Provider, ProviderError, jsonrpc::HttpTransport},
    signers::{LocalWallet, SigningKey},
};
use url::Url;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Args {
    Declare(DeclareCommand),
}

#[derive(Parser, Debug)]
struct DeclareCommand {
    #[arg(short('i'), long)]
    chain_id: String,
    #[arg(short, long)]
    url: String,
    #[arg(short, long, value_hint(ValueHint::FilePath))]
    sierra_json: PathBuf,
    #[arg(short, long)]
    compiled_class_hash: String,
    #[arg(short, long)]
    private_key: String,
    #[arg(short, long)]
    account_address: String,
}

fn init_account(
    cmd: &DeclareCommand,
) -> Result<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>, Error> {
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(Felt::from_hex(
        &cmd.private_key,
    )?));
    let address = Felt::from_hex(&cmd.account_address)?;

    let provider = JsonRpcClient::new(HttpTransport::new(Url::parse(&cmd.url)?));

    let account = SingleOwnerAccount::new(
        provider,
        signer,
        address,
        Felt::from_bytes_be_slice(cmd.chain_id.as_bytes()),
        ExecutionEncoding::New,
    );

    Ok(account)
}

// cargo run -p starknet-on-chain-setup -- declare
// --network=local
// --sierra-json=./contracts/invoice/target/release/invoice_payment_InvoicePayment.contract_class.json
// --compiled-class-hash=0x01fcc070469e43efcb1e4a71243dcdefce8f2e1bfdba5052aa233bb8383aec38
// --private-key=0x0000000000000000000000000000000071d7bb07b9a64f6f78ac4c816aff4da9
// --account-address=0x064b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let args = Args::parse();

    match args {
        Args::Declare(declare_command) => declare(declare_command).await?,
    }

    Ok(())
}

async fn declare(cmd: DeclareCommand) -> Result<(), Error> {
    let compiled_class_hash = Felt::from_hex(&cmd.compiled_class_hash)?;

    let contract_artifact: SierraClass =
        serde_json::from_reader(std::fs::File::open(&cmd.sierra_json)?)?;

    let flattened_class = contract_artifact.flatten()?;
    let class_hash = flattened_class.class_hash();

    let account = init_account(&cmd)?;

    if let Err(ProviderError::StarknetError(StarknetError::ClassHashNotFound)) = account
        .provider()
        .get_class(BlockId::Tag(BlockTag::Latest), class_hash)
        .await
    {
        let declare_result = account
            .declare_v3(Arc::new(flattened_class), compiled_class_hash)
            .send()
            .await?;
        info!("declare tx hash: {:#064x}", declare_result.transaction_hash);
        watch_tx(account.provider(), declare_result.transaction_hash).await?;
        let current_block = account.provider().block_number().await?;
        while account.provider().block_number().await? == current_block {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        info!("declared class hash: {:#064x}", declare_result.class_hash);
    } else {
        debug!("class already declared");
    };

    let contract_factory = ContractFactory::new(class_hash, account);
    let deploy_tx = contract_factory.deploy_v3(vec![], Felt::ZERO, false);
    let contract_address = deploy_tx.deployed_address();
    let deploy_result = deploy_tx.send().await?;
    info!("deploy tx hash: {:#064x}", deploy_result.transaction_hash);
    info!("deployed contract address: {:#064x}", contract_address);

    Ok(())
}

pub async fn watch_tx<P>(provider: P, transaction_hash: Felt) -> Result<(), anyhow::Error>
where
    P: Provider,
{
    loop {
        match provider.get_transaction_status(transaction_hash).await {
            Ok(TransactionStatus::AcceptedOnL2(TransactionExecutionStatus::Succeeded)) => {
                return Ok(());
            }
            Ok(TransactionStatus::AcceptedOnL2(TransactionExecutionStatus::Reverted)) => {
                return Err(anyhow!("tx reverted"));
            }
            Ok(TransactionStatus::Received) => {}
            Ok(TransactionStatus::Rejected) => return Err(anyhow!("tx rejected")),
            Err(ProviderError::StarknetError(StarknetError::TransactionHashNotFound)) => {}
            Err(err) => return Err(err.into()),
            Ok(TransactionStatus::AcceptedOnL1(_)) => unreachable!(),
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
