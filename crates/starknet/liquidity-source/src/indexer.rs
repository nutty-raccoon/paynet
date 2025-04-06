use crate::StarknetCliConfig;
use futures::TryStreamExt;
use nuts::Amount;
use nuts::nut04::MintQuoteState;
use sqlx::{PgConnection, Postgres, pool::PoolConnection};
use starknet_payment_indexer::{ApibaraIndexerService, Message, PaymentEvent, Uri};
use starknet_types::{Asset, ChainId};
use starknet_types::{StarknetU256, Unit::MilliStrk};
use starknet_types_core::felt::Felt;
use std::env;
use std::str::FromStr;
use tokio::select;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to open connection with sqlite db: {0}")]
    OpenSqlite(#[source] rusqlite::Error),
    #[error("unknown chain id: {0}")]
    UnknownChainId(ChainId),
    #[error("failed to read environment variable `{0}`: {1}")]
    Env(&'static str, #[source] std::env::VarError),
    #[error("failed init apibara indexer: {0}")]
    InitIndexer(#[source] starknet_payment_indexer::Error),
    #[error("failed to interact with the indexer: {0}")]
    Indexer(#[from] anyhow::Error),
    #[error("failed to interact with the node database: {0}")]
    DbNode(#[from] db_node::Error),
    #[error("failed to interact with the node database: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Felt(#[from] starknet_types_core::felt::FromStrError),
    #[error("u256 value overflowed during the computation of the total amount paid for invoice")]
    AmountPaidOverflow,
    #[error(transparent)]
    StarknetU256ToAmount(#[from] starknet_types::StarknetU256ToAmountError),
}

async fn init_indexer_task(
    apibara_token: String,
    chain_id: ChainId,
    payee_address: Felt,
) -> Result<ApibaraIndexerService, Error> {
    let conn = rusqlite::Connection::open_in_memory().map_err(Error::OpenSqlite)?;

    let on_chain_constants = starknet_types::constants::ON_CHAIN_CONSTANTS
        .get(chain_id.as_str())
        .ok_or(Error::UnknownChainId(chain_id))?;
    let strk_token_address = on_chain_constants
        .assets_contract_address
        .get(Asset::Strk.as_str())
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
        vec![(payee_address, *strk_token_address)],
    )
    .await
    .map_err(Error::InitIndexer)?;

    Ok(service)
}

async fn listen_to_indexer(
    mut db_conn: PoolConnection<Postgres>,
    mut indexer_service: ApibaraIndexerService,
) -> Result<(), Error> {
    while let Some(event) = indexer_service.try_next().await? {
        match event {
            Message::Payment(payment_events) => {
                process_payment_event(payment_events, &mut db_conn).await?;
            }
            Message::Invalidate {
                last_valid_block_number: _,
                last_valid_block_hash: _,
            } => {
                todo!();
            }
        }
    }

    Ok(())
}

pub async fn run_in_ctrl_c_cancellable_task(
    db_conn: PoolConnection<Postgres>,
    apibara_token: String,
    config: &StarknetCliConfig,
) -> Result<(), Error> {
    let indexer_service = init_indexer_task(
        apibara_token,
        config.chain_id.clone(),
        config.our_account_address,
    )
    .await?;

    let _indexer_handle = tokio::spawn(async move {
        select! {
            indexer_res = listen_to_indexer(db_conn, indexer_service) => match indexer_res {
                Ok(()) => {},
                Err(err) => eprintln!("indexer task failed: {}", err),
            },
            sig = tokio::signal::ctrl_c() => match sig {
                Ok(()) => {},
                Err(err) => eprintln!("unable to listen for shutdown signal: {}", err)
            }
        }
    });

    Ok(())
}

async fn process_payment_event(
    payment_events: Vec<PaymentEvent>,
    db_conn: &mut PgConnection,
) -> Result<(), Error> {
    for payment_event in payment_events {
        let quote_id = match db_node::mint_quote::get_quote_id_by_invoice_id(
            db_conn,
            &payment_event.invoice_id.to_bytes_be(),
        )
        .await?
        {
            // TODO: also check if it exists in the metl quote table.
            // If so, set the quote state to paid
            None => continue,
            Some(mint_quote_id) => mint_quote_id,
        };
        db_node::payment_event::insert_new_payment_event(db_conn, &payment_event).await?;
        let current_paid = db_node::payment_event::get_current_paid(
            db_conn,
            &payment_event.invoice_id.to_bytes_be(),
        )
        .await?
        .map(|(low, high)| -> Result<primitive_types::U256, Error> {
            let amount_as_strk_256 = StarknetU256 {
                low: Felt::from_str(&low)?,
                high: Felt::from_str(&high)?,
            };

            Ok(primitive_types::U256::from(amount_as_strk_256))
        })
        .try_fold(primitive_types::U256::zero(), |acc, a| match a {
            Ok(v) => v.checked_add(acc).ok_or(Error::AmountPaidOverflow),
            Err(e) => Err(e),
        })?;

        let quote_expected_amount = db_node::mint_quote::get_amount_from_invoice_id(
            db_conn,
            &payment_event.invoice_id.to_bytes_be(),
        )
        .await?;

        let current_paid_starknet_u256: StarknetU256 = current_paid.into();

        let current_paid_amount = MilliStrk
            .convert_u256_into_amount(current_paid_starknet_u256)
            .map(|(a, _r)| a)?;

        if current_paid_amount >= Amount::from(quote_expected_amount) {
            db_node::mint_quote::set_state(db_conn, quote_id, MintQuoteState::Paid).await?;
        }
    }

    Ok(())
}
