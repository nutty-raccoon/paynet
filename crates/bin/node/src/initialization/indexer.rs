use std::env;

use crate::indexer;
use sqlx::{Pool, Postgres};
use starknet_payment_indexer::{ApibaraIndexerService, Uri};
use starknet_types::{Asset, ChainId};
use starknet_types_core::felt::Felt;

use super::{Error, StarknetConfig};

async fn init_indexer_task(
    apibara_token: String,
    chain_id: ChainId,
    recipient_address: Felt,
) -> Result<ApibaraIndexerService, Error> {
    let conn = rusqlite::Connection::open_in_memory().map_err(Error::OpenSqlite)?;

    let on_chain_constants = starknet_types::constants::ON_CHAIN_CONSTANTS
        .get(chain_id.as_ref())
        .ok_or(Error::UnknownChainId(chain_id))?;
    let strk_token_address = on_chain_constants
        .assets_contract_address
        .get(Asset::Strk.as_ref())
        .expect("asset 'strk' should be part of the constants");

    let uri = match on_chain_constants.apibara.data_stream_uri {
        Some(uri) => starknet_payment_indexer::Uri::from_static(uri),
        None => env::var("DNA_URI")
            .map_err(|e| Error::Env("DNA_URI", e))?
            .parse::<Uri>()
            .map_err(|e| Error::InitIndexer(starknet_payment_indexer::Error::ParseURI(e)))?,
    };
    let service = starknet_payment_indexer::ApibaraIndexerService::init(
        conn,
        apibara_token,
        uri,
        on_chain_constants.apibara.starting_block,
        vec![(recipient_address, *strk_token_address)],
    )
    .await
    .map_err(Error::InitIndexer)?;

    Ok(service)
}

pub async fn launch_indexer_task(
    pg_pool: &Pool<Postgres>,
    apibara_token: String,
    config: &StarknetConfig,
) -> Result<impl Future<Output = Result<(), crate::Error>>, crate::Error> {
    let indexer_service = init_indexer_task(
        apibara_token,
        config.chain_id.clone(),
        config.recipient_address,
    )
    .await?;
    let db_conn = pg_pool.acquire().await?;
    let indexer_future = indexer::listen_to_indexer(db_conn, indexer_service);

    Ok(indexer_future)
}
