use std::{
    hash::{DefaultHasher, Hash, Hasher},
    str::FromStr,
    time::Duration,
};

use nuts::{nut04::MintQuoteState, traits::Unit as UnitT};
use starknet_types::{Unit, UnitFromStrError};
use tauri::{AppHandle, Manager, async_runtime};
use tokio::sync::mpsc;
use tracing::{Level, error, event};
use wallet::{
    db::{self, mint_quote::MintQuote},
    mint::RedeemQuoteError,
};

use crate::{
    AppState,
    errors::CommonError,
    front_events::{
        BalanceChange, MintQuoteIdentifier, MintQuotePaidEvent, MintQuoteRedeemedEvent,
        RemoveMintQuoteEvent, emit_balance_increase_event, emit_mint_quote_paid_event,
        emit_mint_quote_redeemed_event, emit_remove_mint_quote_event,
    },
};

#[derive(Debug, Clone)]
pub enum MintQuoteStateMachine {
    Created { node_id: u32, quote_id: String },
    Paid { node_id: u32, quote_id: String },
    Redeemed { node_id: u32, quote_id: String },
}

#[derive(Debug, Clone, PartialEq)]
enum QuoteState {
    Unpaid,
    Paid,
}

pub struct MintQuoteHandler {
    app: AppHandle,
    rx: mpsc::Receiver<MintQuoteStateMachine>,
    quotes: Vec<(u64, QuoteState)>,
}

impl MintQuoteHandler {
    pub fn new(app: AppHandle, rx: mpsc::Receiver<MintQuoteStateMachine>) -> Self {
        Self {
            app,
            rx,
            quotes: Vec::new(),
        }
    }

    fn hash_quote(node_id: u32, quote_id: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        (node_id, quote_id).hash(&mut hasher);
        hasher.finish()
    }

    fn find_quote_state(&self, hash: u64) -> Option<&QuoteState> {
        self.quotes
            .iter()
            .find(|(h, _)| *h == hash)
            .map(|(_, state)| state)
    }

    fn update_quote_state(&mut self, hash: u64, new_state: QuoteState) {
        if let Some(pos) = self.quotes.iter().position(|(h, _)| *h == hash) {
            self.quotes[pos].1 = new_state;
        } else {
            self.quotes.push((hash, new_state));
        }
    }
    fn remove_quote_state(&mut self, hash: u64) {
        if let Some(pos) = self.quotes.iter().position(|(h, _)| *h == hash) {
            self.quotes.remove(pos);
        }
    }
}

impl MintQuoteHandler {
    pub async fn run(mut self) {
        while let Some(event) = self.rx.recv().await {
            let cloned_app = self.app.clone();
            match event {
                MintQuoteStateMachine::Created { node_id, quote_id } => {
                    let hash = Self::hash_quote(node_id, &quote_id);

                    // Only process if not already unpaid
                    if self.find_quote_state(hash) != Some(&QuoteState::Unpaid) {
                        self.update_quote_state(hash, QuoteState::Unpaid);
                        async_runtime::spawn(async move {
                            sync_quote_until_is_paid(cloned_app, node_id, quote_id)
                                .await
                                .inspect_err(|e| error!("failed to sync unpaid mint quote: {e}"))
                        });
                    }
                }
                MintQuoteStateMachine::Paid { node_id, quote_id } => {
                    let hash = Self::hash_quote(node_id, &quote_id);

                    // Only process if not already paid
                    if self.find_quote_state(hash) != Some(&QuoteState::Paid) {
                        self.update_quote_state(hash, QuoteState::Paid);
                        async_runtime::spawn(async move {
                            try_redeem_quote(cloned_app, node_id, quote_id)
                                .await
                                .inspect_err(|e| error!("failed to redeem mint quote: {e}"))
                        });
                    }
                }
                MintQuoteStateMachine::Redeemed { node_id, quote_id } => {
                    let hash = Self::hash_quote(node_id, &quote_id);

                    // Cleanup
                    self.remove_quote_state(hash);
                }
            }
            // Reclaim memory
            let capacity = self.quotes.capacity();
            if capacity > 10 && capacity > self.quotes.len() * 2 {
                self.quotes.shrink_to(self.quotes.len() * 3 / 2);
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Common(#[from] CommonError),
    #[error("failed to send message through channel")]
    SendMessage,
    #[error("failed to build unit from string: {0}")]
    UnitFromStr(#[from] UnitFromStrError),
    #[error("invalid mint quote state for operation, expected {expected}, got {got}")]
    InvalidQuoteState {
        expected: MintQuoteState,
        got: MintQuoteState,
    },
    #[error("failed to redeem quote {0}: {1}")]
    Redeem(String, #[source] RedeemQuoteError),
}

pub async fn start_syncing_mint_quotes(
    app: AppHandle,
    rx: mpsc::Receiver<MintQuoteStateMachine>,
) -> Result<(), Error> {
    let mint_handler = MintQuoteHandler::new(app.clone(), rx);
    let _handle = async_runtime::spawn(mint_handler.run());

    let state = app.state::<AppState>();
    let events_to_send = {
        let conn = state.pool.get().map_err(CommonError::DbPool)?;
        let pending_mint_quotes_by_node =
            wallet::db::mint_quote::get_pendings(&conn).map_err(CommonError::Db)?;

        let mut events = Vec::new();
        for (node_id, pending_mint_quotes) in pending_mint_quotes_by_node {
            for pending_mint_quote in pending_mint_quotes {
                match pending_mint_quote.state {
                    MintQuoteState::Unpaid => events.push(MintQuoteStateMachine::Created {
                        node_id,
                        quote_id: pending_mint_quote.id,
                    }),
                    MintQuoteState::Paid => events.push(MintQuoteStateMachine::Paid {
                        node_id,
                        quote_id: pending_mint_quote.id,
                    }),
                    MintQuoteState::Issued => unreachable!(),
                }
            }
        }
        events
    };
    for event in events_to_send {
        state
            .mint_quote_event_sender
            .send(event)
            .await
            .map_err(|_| Error::SendMessage)?;
    }

    Ok(())
}

pub async fn try_redeem_quote(app: AppHandle, node_id: u32, quote_id: String) -> Result<(), Error> {
    let state = app.state::<AppState>();
    let MintQuote {
        node_id,
        method,
        amount,
        unit,
        state: quote_state,
        ..
    } = db::mint_quote::get(&state.pool.get().unwrap(), node_id, &quote_id)
        .map_err(CommonError::Db)?
        .ok_or(CommonError::QuoteNotFound(quote_id.clone()))?;

    if quote_state != MintQuoteState::Paid {
        return Err(Error::InvalidQuoteState {
            expected: MintQuoteState::Paid,
            got: quote_state,
        });
    }

    let mut node_client = state
        .get_node_client_connection(node_id)
        .await
        .map_err(CommonError::CachedConnection)?;

    wallet::mint::redeem_quote(
        crate::SEED_PHRASE_MANAGER,
        state.pool.clone(),
        &mut node_client,
        method,
        &quote_id,
        node_id,
        unit.as_str(),
        amount,
    )
    .await
    .map_err(|e| Error::Redeem(quote_id.clone(), e))?;
    event!(name: "mint-quote-redeemed", Level::INFO, %quote_id, "Mint quote redeemed");

    let asset = Unit::from_str(&unit)?.matching_asset();
    emit_balance_increase_event(
        &app,
        BalanceChange {
            node_id,
            unit,
            amount: amount.into(),
        },
    )
    .map_err(CommonError::EmitTauriEvent)?;

    state.get_prices_config.write().await.assets.insert(asset);

    state
        .mint_quote_event_sender
        .send(MintQuoteStateMachine::Redeemed {
            node_id,
            quote_id: quote_id.clone(),
        })
        .await
        .map_err(|_| Error::SendMessage)?;

    emit_mint_quote_redeemed_event(
        &app,
        MintQuoteRedeemedEvent(MintQuoteIdentifier {
            node_id,
            quote_id: quote_id.clone(),
        }),
    )
    .map_err(CommonError::EmitTauriEvent)?;

    Ok(())
}

pub async fn sync_quote_until_is_paid(
    app: AppHandle,
    node_id: u32,
    quote_id: String,
) -> Result<(), Error> {
    const LOOP_INTERVAL: Duration = Duration::from_secs(1);

    let state = app.state::<AppState>();
    let mut node_client = state
        .get_node_client_connection(node_id)
        .await
        .map_err(CommonError::CachedConnection)?;
    loop {
        let quote = db::mint_quote::get(&state.pool.get().unwrap(), node_id, &quote_id)
            .map_err(CommonError::Db)?
            .ok_or(CommonError::QuoteNotFound(quote_id.clone()))?;

        let res = wallet::sync::mint_quote(
            state.pool.clone(),
            &mut node_client,
            quote.method,
            quote_id.clone(),
        )
        .await;

        match res {
            Ok(Some(MintQuoteState::Unpaid)) => {}
            Ok(Some(MintQuoteState::Paid)) => {
                event!(name: "mint-quote-paid", Level::INFO, %quote_id, "Mint quote paid");
                emit_mint_quote_paid_event(
                    &app,
                    MintQuotePaidEvent(MintQuoteIdentifier {
                        node_id,
                        quote_id: quote_id.clone(),
                    }),
                )
                .map_err(CommonError::EmitTauriEvent)?;

                state
                    .mint_quote_event_sender
                    .send(MintQuoteStateMachine::Paid {
                        node_id,
                        quote_id: quote_id.clone(),
                    })
                    .await
                    .map_err(|_| Error::SendMessage)?;

                break;
            }
            Ok(Some(MintQuoteState::Issued)) => {
                event!(name: "mint-quote-issued", Level::INFO, %quote_id, "Mint quote issued");
                error!(
                    "mint quote {} has been issued before it was synced as paid",
                    quote_id
                );
                break;
            }
            Ok(None) => {
                event!(name: "mint-quote-expired", Level::INFO, %quote_id, "Mint quote expired");
                emit_remove_mint_quote_event(
                    &app,
                    RemoveMintQuoteEvent(MintQuoteIdentifier { node_id, quote_id }),
                )
                .map_err(CommonError::EmitTauriEvent)?;
                break;
            }
            Err(e) => {
                error!("failed to sync mint quote {}: {}", quote_id, e);
                break;
            }
        }

        tokio::time::sleep(LOOP_INTERVAL).await;
    }
    event!(name: "mint-quote-paid", Level::INFO, "Exiting the loop");

    Ok(())
}
