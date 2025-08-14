use anyhow::{Context, Error, anyhow};
use clap::{Parser, ValueHint};
use env_logger;
use ethabi::{Address as AbiAddress, Token, Uint};
use ethers::{
    abi::Abi,
    contract::ContractFactory,
    core::types::{Address, Bytes, H256, TransactionReceipt, TransactionRequest, U256},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
};
use log::{error, info};
use serde::Deserialize;
use std::{path::PathBuf, sync::Arc, time::Duration};

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
enum Commands {
    Deploy(DeployCommand),
    PayInvoice(PayInvoiceCommand),
}

#[derive(Parser, Debug)]
struct AccountArgs {
    #[arg(long)]
    url: String,
    #[arg(long)]
    chain_id: u64,
    #[arg(long)]
    private_key: String,
}

#[derive(Parser, Debug)]
struct DeployCommand {
    #[arg(long, value_hint(ValueHint::FilePath))]
    abi_json: PathBuf,
    #[arg(long, value_hint(ValueHint::FilePath))]
    bytecode_json_or_hex: PathBuf,
}

#[derive(Parser, Debug)]
struct PayInvoiceCommand {
    #[arg(long)]
    to: String,
    #[arg(long)]
    quote_id_hash: String,
    #[arg(long)]
    expiry: u64,
    #[arg(long)]
    asset: String,
    #[arg(long)]
    amount: String,
    #[arg(long)]
    payee: String,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Cli {
    #[command(flatten)]
    account: AccountArgs,

    #[command(subcommand)]
    command: Commands,
}

type EthClient = SignerMiddleware<Provider<Http>, LocalWallet>;

const PAY_INVOICE_SELECTOR: [u8; 4] = [0xbe, 0x55, 0xe0, 0x30];

async fn init_client(args: &AccountArgs) -> Result<EthClient, Error> {
    let provider = Provider::<Http>::try_from(args.url.as_str())
        .with_context(|| "invalid RPC url")?
        .interval(Duration::from_millis(2000));

    let wallet: LocalWallet = args
        .private_key
        .parse::<LocalWallet>()
        .context("invalid private key")?
        .with_chain_id(args.chain_id);

    let addr = wallet.address();
    let client = SignerMiddleware::new(provider, wallet);
    info!("using account: {addr:?}");
    Ok(client)
}

#[derive(Deserialize)]
struct RawBytecode {
    object: String,
}

#[derive(Deserialize)]
struct Artifact {
    abi: Abi,
    #[serde(default)]
    bytecode: Option<RawBytecode>,
}

pub fn load_bytecode(path: &PathBuf) -> Result<Bytes, Error> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("read bytecode file: {}", path.display()))?;

    if let Ok(raw) = serde_json::from_str::<RawBytecode>(&content) {
        let hex = raw.object;
        return Ok(hex.strip_prefix("0x").unwrap_or(&hex).parse()?);
    }

    if let Ok(art) = serde_json::from_str::<Artifact>(&content) {
        if let Some(bc) = art.bytecode {
            let hex = bc.object;
            return Ok(hex.strip_prefix("0x").unwrap_or(&hex).parse()?);
        }
        return Err(anyhow!("artifact missing bytecode.object"));
    }

    let hex = content.trim();
    Ok(hex.strip_prefix("0x").unwrap_or(hex).parse()?)
}

pub fn load_abi(path: &PathBuf) -> Result<Abi, Error> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("read ABI file: {}", path.display()))?;

    if let Ok(abi) = serde_json::from_str::<Abi>(&content) {
        return Ok(abi);
    }

    let art: Artifact =
        serde_json::from_str(&content).with_context(|| "parse artifact with { abi, bytecode }")?;
    Ok(art.abi)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let cli = Cli::parse();
    let client = init_client(&cli.account).await?;

    match cli.command {
        Commands::Deploy(cmd) => deploy(client, cmd).await?,
        Commands::PayInvoice(cmd) => pay(&client, cmd).await?,
    }

    Ok(())
}

async fn deploy(client: EthClient, cmd: DeployCommand) -> Result<(), Error> {
    let abi = load_abi(&cmd.abi_json)?;
    let bytecode = load_bytecode(&cmd.bytecode_json_or_hex)?;
    let deployer = ContractFactory::new(abi, bytecode, Arc::new(client));
    let contract = deployer.deploy(())?.send().await?;
    let deployed = contract.address();
    info!("deployed contract at: {deployed:?}");
    Ok(())
}

async fn pay(client: &EthClient, cmd: PayInvoiceCommand) -> Result<(), Error> {
    let to = parse_address(&cmd.to).context("invalid --to")?;
    let quote = parse_bytes32(&cmd.quote_id_hash)?;
    let expiry = cmd.expiry;
    let asset = parse_address(&cmd.asset).context("invalid --asset")?;
    let amount = parse_u256(&cmd.amount).context("invalid --amount")?;
    let payee = parse_address(&cmd.payee).context("invalid --payee")?;

    let data = encode_pay_invoice_ethabi(quote, expiry, asset, amount, payee);

    let tx = TransactionRequest::new()
        .to(to)
        .data(data)
        .value(U256::zero());

    let pending = client
        .send_transaction(tx, None)
        .await
        .inspect_err(|e| error!("send payInvoice tx failed: {:?}", e))?;

    info!("payInvoice tx sent: {:#066x}", pending.tx_hash());
    let _receipt = watch_tx(client, pending.tx_hash()).await?;
    info!("payInvoice succeeded");
    Ok(())
}

fn encode_pay_invoice_ethabi(
    quote: [u8; 32],
    expiry: u64,
    asset: Address,
    amount: U256,
    payee: Address,
) -> Bytes {
    // U256 -> big-endian bytes for ethabi::Uint
    let mut amt_be = [0u8; 32];
    amount.to_big_endian(&mut amt_be);

    let tokens = vec![
        Token::FixedBytes(quote.to_vec()),
        Token::Uint(Uint::from(expiry)),
        Token::Address(AbiAddress::from_slice(asset.as_fixed_bytes())),
        Token::Uint(Uint::from_big_endian(&amt_be)),
        Token::Address(AbiAddress::from_slice(payee.as_fixed_bytes())),
    ];

    let encoded = ethabi::encode(&tokens);

    let mut data = Vec::with_capacity(4 + encoded.len());
    data.extend_from_slice(&PAY_INVOICE_SELECTOR);
    data.extend_from_slice(&encoded);
    Bytes::from(data)
}

fn parse_bytes32(s: &str) -> Result<[u8; 32], Error> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let raw = hex::decode(s).context("quote_id_hash must be hex")?;
    if raw.len() != 32 {
        anyhow::bail!("quote_id_hash must be 32 bytes (got {} bytes)", raw.len());
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&raw);
    Ok(out)
}

fn parse_address(s: &str) -> Result<Address, Error> {
    Ok(s.parse::<Address>().context("invalid address")?)
}

fn parse_u256(s: &str) -> Result<U256, Error> {
    if let Some(hex) = s.strip_prefix("0x") {
        Ok(U256::from_str_radix(hex, 16)?)
    } else {
        Ok(U256::from_dec_str(s)?)
    }
}

pub async fn watch_tx<M: Middleware>(
    client: &M,
    tx_hash: H256,
) -> Result<TransactionReceipt, Error> {
    loop {
        match client.get_transaction_receipt(tx_hash).await {
            Ok(Some(r)) => {
                if r.status == Some(1u64.into()) {
                    return Ok(r);
                } else {
                    return Err(anyhow!("tx reverted or failed: {:?}", r.status));
                }
            }
            Ok(None) => {
                // still pending
            }
            Err(e) => {
                return Err(anyhow!("get_transaction_receipt failed: {e}"));
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
