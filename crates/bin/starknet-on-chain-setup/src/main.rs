use std::{path::PathBuf, sync::Arc};

use anyhow::{Error, anyhow};
use clap::{Parser, ValueHint};
use starknet::{
    accounts::{Account, ExecutionEncoding, SingleOwnerAccount},
    contract::ContractFactory,
    core::{
        chain_id,
        types::{Felt, contract::SierraClass},
    },
    providers::{JsonRpcClient, jsonrpc::HttpTransport},
    signers::{LocalWallet, SigningKey},
};
use url::Url;

const BLAST_SEPOLIA_RPC_URL: &str = "https://starknet-sepolia.public.blastapi.io/rpc/v0_7";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Args {
    Declare(DeclareCommand),
}

#[derive(Parser, Debug)]
struct DeclareCommand {
    #[arg(short, long)]
    network: String,
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

    let url = match cmd.network.as_str() {
        "sepolia" => BLAST_SEPOLIA_RPC_URL,
        "local" => "http://127.0.0.1:5050",
        _ => return Err(anyhow!("unknown network {}", cmd.network)),
    };
    let provider = JsonRpcClient::new(HttpTransport::new(Url::parse(url)?));

    let account = SingleOwnerAccount::new(
        provider,
        signer,
        address,
        chain_id::SEPOLIA,
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

    let account = init_account(&cmd)?;
    let declare_result = account
        .declare_v3(Arc::new(flattened_class), compiled_class_hash)
        .send()
        .await?;
    println!(
        "Declare transaction hash: {:#064x}",
        declare_result.transaction_hash
    );
    println!("Class hash: {:#064x}", declare_result.class_hash);

    let contract_factory = ContractFactory::new(declare_result.class_hash, account);
    let deploy_tx = contract_factory.deploy_v3(vec![], Felt::ZERO, false);
    let contract_address = deploy_tx.deployed_address();
    let deploy_result = deploy_tx.send().await?;
    println!(
        "Deploy transaction hash: {:#064x}",
        deploy_result.transaction_hash
    );
    println!("Contract address: {:#064x}", contract_address);

    Ok(())
}
