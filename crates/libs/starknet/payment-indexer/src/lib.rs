use std::ops::DerefMut;
use std::task::Poll;

use apibara_core::node::v1alpha2::DataFinality;
use apibara_core::starknet::v1alpha2::{Block, FieldElement, Filter, HeaderFilter};
pub use apibara_sdk::Uri;
use apibara_sdk::{ClientBuilder, Configuration, DataMessage, InvalidUri};
use futures::StreamExt;
use rusqlite::Connection;
use starknet_core::types::Felt;
use starknet_types::constants::ON_CHAIN_CONSTANTS;
use starknet_types::{ChainId, StarknetU256};
use thiserror::Error;

mod db;

const REMITTANCE_EVENT_KEY: &str =
    "0x027a12f554d018764f982295090da45b4ff0734785be0982b62c329b9ac38033";

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid value for field element: {0}")]
    InvalidFieldElement(String),
    #[error("DNA client error: {0}")]
    ApibaraClient(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error(transparent)]
    Db(#[from] rusqlite::Error),
    #[error(transparent)]
    ParseURI(#[from] InvalidUri),
    #[error("unknown chain id: {0}")]
    UnknownChainId(ChainId),
}

pub struct ApibaraIndexerService {
    stream: apibara_sdk::ImmutableDataStream<Block>,
    db_conn: Connection,
}

impl Unpin for ApibaraIndexerService {}

impl ApibaraIndexerService {
    pub async fn init(
        mut db_conn: Connection,
        apibara_bearer_token: String,
        uri: Uri,
        chain_id: ChainId,
        starting_block: u64,
        target_asset_and_payee_pairs: Vec<Felt>,
    ) -> Result<Self, Error> {
        db::create_tables(&mut db_conn)?;

        let on_chain_constants = ON_CHAIN_CONSTANTS
            .get(chain_id.as_str())
            .ok_or(Error::UnknownChainId(chain_id))?;
        let invoice_payment_contract_address = on_chain_constants.invoice_payment_contract_address;

        let config = Configuration::<Filter>::default()
            .with_starting_block(starting_block)
            .with_finality(DataFinality::DataStatusAccepted)
            .with_filter(|mut filter| {
                let remittance_event_key = FieldElement::from_hex(REMITTANCE_EVENT_KEY).unwrap();

                target_asset_and_payee_pairs.iter().for_each(|asset| {
                    filter
                        .with_header(HeaderFilter::weak())
                        .add_event(|event| {
                            event
                                .with_from_address(FieldElement::from_bytes(
                                    &invoice_payment_contract_address.to_bytes_be(),
                                ))
                                .with_keys(vec![
                                    remittance_event_key.clone(),
                                    FieldElement::from_hex(&asset.to_hex_string()).unwrap(),
                                ])
                        })
                        .build();
                });

                filter
            });

        let stream = ClientBuilder::default()
            .with_bearer_token(Some(apibara_bearer_token))
            .connect(uri)
            .await
            .map_err(|e| Error::ApibaraClient(Box::new(e.into_error())))?
            .start_stream_immutable::<Filter, Block>(config)
            .await
            .map_err(|e| Error::ApibaraClient(Box::new(e.into_error())))?;

        Ok(Self { stream, db_conn })
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Payment(Vec<PaymentEvent>),
    Invalidate {
        last_valid_block_number: u64,
        last_valid_block_hash: Vec<u8>,
    },
}

#[derive(Debug, Clone)]
pub struct PaymentEvent {
    pub block_id: String,
    pub tx_hash: Felt,
    pub event_idx: u64,
    pub asset: Felt,
    pub payee: Felt,
    pub invoice_id: Felt,
    pub payer: Felt,
    pub amount: StarknetU256,
}

impl futures::Stream for ApibaraIndexerService {
    type Item = anyhow::Result<Message>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let s = self.deref_mut();

        match s.stream.poll_next_unpin(cx) {
            Poll::Ready(Some(res)) => match res {
                Ok(message) => match message {
                    DataMessage::Data {
                        cursor: _cursor,
                        end_cursor: _end_cursor,
                        finality: _finality,
                        batch,
                    } => {
                        let tx = match s.db_conn.transaction() {
                            Ok(tx) => tx,
                            Err(e) => return Poll::Ready(Some(Err(e.into()))),
                        };

                        let mut payment_events = Vec::with_capacity(batch.len());

                        for block in batch.iter() {
                            let block_infos = block.header.as_ref().unwrap().into();
                            db::insert_new_block(&tx, &block_infos)?;

                            for event in block.events.iter() {
                                let tx_hash = event
                                    .transaction
                                    .as_ref()
                                    .unwrap()
                                    .meta
                                    .as_ref()
                                    .unwrap()
                                    .hash
                                    .as_ref()
                                    .unwrap()
                                    .to_string();

                                let payment_event = match event.event.as_ref().unwrap().try_into() {
                                    Ok(pe) => pe,
                                    Err(e) => {
                                        return Poll::Ready(Some(Err(anyhow::Error::from(e))));
                                    }
                                };
                                db::insert_payment_event(
                                    &tx,
                                    &block_infos.id,
                                    &tx_hash,
                                    &payment_event,
                                )?;
                                payment_events.push(PaymentEvent {
                                    block_id: block_infos.id.clone(),
                                    tx_hash: Felt::from_hex(&tx_hash).unwrap(),
                                    event_idx: payment_event.index,
                                    payee: Felt::from_hex(&payment_event.payee).unwrap(),
                                    payer: Felt::from_hex(&payment_event.payer).unwrap(),
                                    asset: Felt::from_hex(&payment_event.asset).unwrap(),
                                    invoice_id: Felt::from_hex(&payment_event.invoice_id).unwrap(),
                                    amount: StarknetU256::from_parts(
                                        u128::from_str_radix(&payment_event.amount_low[2..], 16)
                                            .unwrap(),
                                        u128::from_str_radix(&payment_event.amount_high[2..], 16)
                                            .unwrap(),
                                    ),
                                });
                            }
                        }

                        match tx.commit() {
                            Ok(()) => Poll::Ready(Some(Ok(Message::Payment(payment_events)))),
                            Err(e) => Poll::Ready(Some(Err(e.into()))),
                        }
                    }
                    DataMessage::Invalidate { cursor } => {
                        let cursor = cursor.unwrap();
                        match db::invalidate(&s.db_conn, cursor.order_key) {
                            Ok(_) => Poll::Ready(Some(Ok(Message::Invalidate {
                                last_valid_block_number: cursor.order_key,
                                last_valid_block_hash: cursor.unique_key,
                            }))),
                            Err(e) => Poll::Ready(Some(Err(e.into()))),
                        }
                    }
                    DataMessage::Heartbeat => Poll::Pending,
                },
                Err(e) => Poll::Ready(Some(Err(anyhow::Error::from(e.into_error())))),
            },
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
