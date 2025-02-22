use futures::TryStreamExt;
use starknet_payment_indexer::ApibaraIndexerService;
use starknet_types_core::felt::Felt;

const APIBARA_TOKEN_ENV_VAR: &str = "APIBARA_TOKEN";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    dotenvy::from_filename("indexer.env")?;
    
    let dna_token =
        std::env::var(APIBARA_TOKEN_ENV_VAR).expect("missing `APIBARA_TOKEN` env variable");

    let starknet_token_address = Felt::from_hex_unchecked(
        "0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d",
    );
    let our_recipient_account = Felt::from_hex_unchecked(
        "0x07487f6e8fc8c60049e82cf8b6593211aeefef7efd0021db585c7e78cc29ac9a",
    );

    let conn = rusqlite::Connection::open_in_memory()?;
    let mut indexer_service = ApibaraIndexerService::init(conn, dna_token, vec![(
        our_recipient_account,
        starknet_token_address,
    )])
    .await?;

    while let Some(event) = indexer_service.try_next().await? {
        println!("{:#?}", event);
        // Do nothing more
        // the indexer is already writing the events in db
    }

    Ok(())
}
