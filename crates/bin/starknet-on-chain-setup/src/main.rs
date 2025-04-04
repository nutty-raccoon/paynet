use std::{path::PathBuf, sync::Arc, time::Duration};

use anyhow::{Error, anyhow};
use clap::{Parser, ValueHint};
use log::{debug, error, info};
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
use starknet_types::Call;
use url::Url;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Commands {
    Declare(DeclareCommand),
    PayInvoice(PayInvoiceCommand),
}

#[derive(Parser, Debug)]
struct AccountArgs {
    #[arg(long)]
    url: String,
    #[arg(long)]
    chain_id: String,
    #[arg(long)]
    private_key: String,
    #[arg(long)]
    account_address: String,
}

#[derive(Parser, Debug)]
struct DeclareCommand {
    #[arg(long, value_hint(ValueHint::FilePath))]
    sierra_json: PathBuf,
    #[arg(long)]
    compiled_class_hash: String,
}

#[derive(Parser, Debug)]
struct PayInvoiceCommand {
    #[arg(long)]
    invoice_json_string: String,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    account: AccountArgs,
    #[command(subcommand)]
    command: Commands,
}

fn init_account(
    account_args: AccountArgs,
) -> Result<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>, Error> {
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(Felt::from_hex(
        &account_args.private_key,
    )?));
    let address = Felt::from_hex(&account_args.account_address)?;

    let provider = JsonRpcClient::new(HttpTransport::new(Url::parse(&account_args.url)?));

    let account = SingleOwnerAccount::new(
        provider,
        signer,
        address,
        Felt::from_bytes_be_slice(account_args.chain_id.as_bytes()),
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

    let cli = Cli::parse();
    let account = init_account(cli.account)?;

    match cli.command {
        Commands::Declare(declare_command) => declare(&account, declare_command).await?,
        Commands::PayInvoice(pay_invoice_command) => pay(&account, pay_invoice_command).await?,
    }

    Ok(())
}

async fn pay(
    account: &SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    cmd: PayInvoiceCommand,
) -> Result<(), Error> {
    let calls: [Call; 2] = serde_json::from_str(&cmd.invoice_json_string)?;

    let tx_hash = account
        .execute_v3(calls.into_iter().map(Into::into).collect())
        .send()
        .await
        .inspect_err(|e| error!("send payment tx failed: {:?}", e))?
        .transaction_hash;

    info!("payment tx sent: {:#064x}", tx_hash);

    watch_tx(account.provider(), tx_hash).await?;
    info!("payment tx succeeded");

    Ok(())
}

async fn declare(
    account: &SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
    cmd: DeclareCommand,
) -> Result<(), Error> {
    let compiled_class_hash = Felt::from_hex(&cmd.compiled_class_hash)?;

    let contract_artifact: SierraClass =
        serde_json::from_reader(std::fs::File::open(&cmd.sierra_json)?)?;

    let flattened_class = contract_artifact.flatten()?;
    let class_hash = flattened_class.class_hash();

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
