use std::{
    hash::{DefaultHasher, Hash, Hasher},
    str::FromStr,
    time::Duration,
};

use nuts::{nut04::MintQuoteState, nut05::MeltQuoteState, traits::Unit as UnitT};
use starknet_types::{Unit, UnitFromStrError};
use tauri::{AppHandle, Manager, async_runtime};
use tokio::sync::mpsc;
use tracing::{Level, error, event};
use wallet::{
    db::{self, melt_quote::MeltQuote, mint_quote::MintQuote},
    melt::PayMeltQuoteError,
    mint::RedeemQuoteError,
};

use crate::{
    AppState,
    errors::CommonError,
    front_events::{
        QuoteIdentifier,
        balance_events::{BalanceChange, emit_balance_decrease_event, emit_balance_increase_event},
        melt_quote_events::{
            emit_melt_quote_paid_event, emit_melt_quote_redeemed_event,
            emit_remove_melt_quote_event,
        },
        mint_quote_events::{
            emit_mint_quote_paid_event, emit_mint_quote_redeemed_event,
            emit_remove_mint_quote_event,
        },
    },
};

#[derive(Debug, Clone)]
pub enum MintQuoteAction {
    Pay { node_id: u32, quote_id: String },
    Redeem { node_id: u32, quote_id: String },
    Done { node_id: u32, quote_id: String },
}

#[derive(Debug, Clone)]
pub enum MeltQuoteAction {
    Pay { node_id: u32, quote_id: String },
    WaitOnChainPayment { node_id: u32, quote_id: String },
    Done { node_id: u32, quote_id: String },
}

#[derive(Debug, Clone)]
pub enum QuoteHandlerEvent {
    Mint(MintQuoteAction),
    Melt(MeltQuoteAction),
}

#[derive(Debug, Clone, PartialEq)]
enum MintState {
    Created,
    Paid,
}

#[derive(Debug, Clone, PartialEq)]
enum MeltState {
    Unpaid,
    Pending,
}

pub struct QuoteHandler {
    app: AppHandle,
    rx: mpsc::Receiver<QuoteHandlerEvent>,
    mint_quotes: Vec<(u64, MintState)>,
    melt_quotes: Vec<(u64, MeltState)>,
}

impl QuoteHandler {
    pub fn new(app: AppHandle, rx: mpsc::Receiver<QuoteHandlerEvent>) -> Self {
        Self {
            app,
            rx,
            mint_quotes: Vec::new(),
            melt_quotes: Vec::new(),
        }
    }

    fn hash_quote(node_id: u32, quote_id: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        (node_id, quote_id).hash(&mut hasher);
        hasher.finish()
    }

    fn find_mint_quote_state(&self, hash: u64) -> Option<&MintState> {
        self.mint_quotes
            .iter()
            .find(|(h, _)| *h == hash)
            .map(|(_, state)| state)
    }

    fn find_melt_quote_state(&self, hash: u64) -> Option<&MeltState> {
        self.melt_quotes
            .iter()
            .find(|(h, _)| *h == hash)
            .map(|(_, state)| state)
    }

    fn update_mint_quote_state(&mut self, hash: u64, new_state: MintState) {
        if let Some(pos) = self.mint_quotes.iter().position(|(h, _)| *h == hash) {
            self.mint_quotes[pos].1 = new_state;
        } else {
            self.mint_quotes.push((hash, new_state));
        }
    }

    fn update_melt_quote_state(&mut self, hash: u64, new_state: MeltState) {
        if let Some(pos) = self.melt_quotes.iter().position(|(h, _)| *h == hash) {
            self.melt_quotes[pos].1 = new_state;
        } else {
            self.melt_quotes.push((hash, new_state));
        }
    }

    fn remove_mint_quote_state(&mut self, hash: u64) {
        if let Some(pos) = self.mint_quotes.iter().position(|(h, _)| *h == hash) {
            self.mint_quotes.remove(pos);
        }
    }

    fn remove_melt_quote_state(&mut self, hash: u64) {
        if let Some(pos) = self.melt_quotes.iter().position(|(h, _)| *h == hash) {
            self.melt_quotes.remove(pos);
        }
    }

    fn shrink_collections(&mut self) {
        // Reclaim memory for mint quotes
        let mint_capacity = self.mint_quotes.capacity();
        if mint_capacity > 10 && mint_capacity > self.mint_quotes.len() * 2 {
            self.mint_quotes.shrink_to(self.mint_quotes.len() * 3 / 2);
        }

        // Reclaim memory for melt quotes
        let melt_capacity = self.melt_quotes.capacity();
        if melt_capacity > 10 && melt_capacity > self.melt_quotes.len() * 2 {
            self.melt_quotes.shrink_to(self.melt_quotes.len() * 3 / 2);
        }
    }
}

impl QuoteHandler {
    pub async fn run(mut self) {
        while let Some(event) = self.rx.recv().await {
            match event {
                QuoteHandlerEvent::Mint(mint_event) => {
                    self.handle_mint_event(mint_event).await;
                }
                QuoteHandlerEvent::Melt(melt_event) => {
                    self.handle_melt_event(melt_event).await;
                }
            }

            self.shrink_collections();
        }
    }

    async fn handle_mint_event(&mut self, event: MintQuoteAction) {
        match event {
            MintQuoteAction::Pay { node_id, quote_id } => {
                let hash = Self::hash_quote(node_id, &quote_id);

                // Only process if not already created
                if self.find_mint_quote_state(hash) != Some(&MintState::Created) {
                    self.update_mint_quote_state(hash, MintState::Created);
                    let app = self.app.clone();
                    async_runtime::spawn(async move {
                        sync_mint_quote_until_is_paid(app, node_id, quote_id)
                            .await
                            .inspect_err(|e| error!("failed to sync unpaid mint quote: {e}"))
                    });
                }
            }
            MintQuoteAction::Redeem { node_id, quote_id } => {
                let hash = Self::hash_quote(node_id, &quote_id);

                // Only process if not already paid
                if self.find_mint_quote_state(hash) != Some(&MintState::Paid) {
                    self.update_mint_quote_state(hash, MintState::Paid);
                    let app = self.app.clone();
                    async_runtime::spawn(async move {
                        try_redeem_mint_quote(app, node_id, quote_id)
                            .await
                            .inspect_err(|e| error!("failed to redeem mint quote: {e}"))
                    });
                }
            }
            MintQuoteAction::Done { node_id, quote_id } => {
                let hash = Self::hash_quote(node_id, &quote_id);
                // Cleanup
                self.remove_mint_quote_state(hash);
            }
        }
    }

    async fn handle_melt_event(&mut self, event: MeltQuoteAction) {
        match event {
            MeltQuoteAction::Pay { node_id, quote_id } => {
                let hash = Self::hash_quote(node_id, &quote_id);

                // Only process if not already unpaid
                if self.find_melt_quote_state(hash) != Some(&MeltState::Unpaid) {
                    self.update_melt_quote_state(hash, MeltState::Unpaid);
                    let app = self.app.clone();
                    async_runtime::spawn(async move {
                        try_pay_melt_quote(app, node_id, quote_id)
                            .await
                            .inspect_err(|e| error!("failed to sync unpaid melt quote: {e}"))
                    });
                }
            }
            MeltQuoteAction::WaitOnChainPayment { node_id, quote_id } => {
                let hash = Self::hash_quote(node_id, &quote_id);

                // Only process if not already pending
                if self.find_melt_quote_state(hash) != Some(&MeltState::Pending) {
                    self.update_melt_quote_state(hash, MeltState::Pending);
                    let app = self.app.clone();
                    async_runtime::spawn(async move {
                        sync_melt_quote_until_is_paid(app, node_id, quote_id)
                            .await
                            .inspect_err(|e| error!("failed to sync pending melt quote: {e}"))
                    });
                }
            }
            MeltQuoteAction::Done { node_id, quote_id } => {
                let hash = Self::hash_quote(node_id, &quote_id);
                // Cleanup
                self.remove_melt_quote_state(hash);
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
    InvalidMintQuoteState {
        expected: MintQuoteState,
        got: MintQuoteState,
    },
    #[error("invalid melt quote state for operation, expected {expected}, got {got}")]
    InvalidMeltQuoteState {
        expected: MeltQuoteState,
        got: MeltQuoteState,
    },
    #[error("failed to redeem mint quote {0}: {1}")]
    RedeemMintQuote(String, #[source] RedeemQuoteError),
    #[error("failed to pay melt quote {0}: {1}")]
    PayMeltQuote(String, #[source] PayMeltQuoteError),
    #[error("failed to wait for payment of melt quote: {0}")]
    WaitForMeltQuotePayment(wallet::errors::Error),
}

pub async fn start_syncing_quotes(
    app: AppHandle,
    rx: mpsc::Receiver<QuoteHandlerEvent>,
) -> Result<(), Error> {
    let quote_handler = QuoteHandler::new(app.clone(), rx);
    let _handle = async_runtime::spawn(quote_handler.run());

    let state = app.state::<AppState>();
    let events_to_send = {
        let conn = state.pool.get().map_err(CommonError::DbPool)?;

        // Get pending mint quotes
        let pending_mint_quotes_by_node =
            wallet::db::mint_quote::get_pendings(&conn).map_err(CommonError::Db)?;

        // Get pending melt quotes
        let pending_melt_quotes_by_node =
            wallet::db::melt_quote::get_pendings(&conn).map_err(CommonError::Db)?;

        let mut events = Vec::new();

        // Process mint quotes
        for (node_id, pending_mint_quotes) in pending_mint_quotes_by_node {
            for pending_mint_quote in pending_mint_quotes {
                let mint_event = match pending_mint_quote.state {
                    MintQuoteState::Unpaid => MintQuoteAction::Pay {
                        node_id,
                        quote_id: pending_mint_quote.id,
                    },
                    MintQuoteState::Paid => MintQuoteAction::Redeem {
                        node_id,
                        quote_id: pending_mint_quote.id,
                    },
                    MintQuoteState::Issued => continue, // Skip issued quotes
                };
                events.push(QuoteHandlerEvent::Mint(mint_event));
            }
        }

        // Process melt quotes
        for (node_id, pending_melt_quotes) in pending_melt_quotes_by_node {
            for pending_melt_quote in pending_melt_quotes {
                let melt_event = match pending_melt_quote.state {
                    MeltQuoteState::Unpaid => MeltQuoteAction::Pay {
                        node_id,
                        quote_id: pending_melt_quote.id,
                    },
                    MeltQuoteState::Pending => MeltQuoteAction::WaitOnChainPayment {
                        node_id,
                        quote_id: pending_melt_quote.id,
                    },
                    MeltQuoteState::Paid => continue, // Skip paid quotes as they're complete
                };
                events.push(QuoteHandlerEvent::Melt(melt_event));
            }
        }

        events
    };

    for event in events_to_send {
        state
            .quote_event_sender
            .send(event)
            .await
            .map_err(|_| Error::SendMessage)?;
    }

    Ok(())
}

#[tracing::instrument(skip(app))]
pub async fn try_redeem_mint_quote(
    app: AppHandle,
    node_id: u32,
    quote_id: String,
) -> Result<(), Error> {
    let state = app.state::<AppState>();
    let MintQuote {
        node_id,
        method,
        amount,
        unit,
        state: quote_state,
        ..
    } = {
        let pool = state.pool.get().map_err(CommonError::DbPool)?;
        db::mint_quote::get(&pool, node_id, &quote_id)
            .map_err(CommonError::Db)?
            .ok_or(CommonError::QuoteNotFound(quote_id.clone()))?
    };

    if quote_state != MintQuoteState::Paid {
        event!(name: "cannot_redeem_unpaid_quote", Level::WARN,
            node_id = node_id,
            quote_id = %quote_id,
            quote_state = ?quote_state
        );
        return Err(Error::InvalidMintQuoteState {
            expected: MintQuoteState::Paid,
            got: quote_state,
        });
    }

    let mut node_client = state
        .get_node_client_connection(node_id)
        .await
        .map_err(CommonError::CachedConnection)?;

    event!(name: "redeeming_mint_quote_with_node", Level::INFO,
        node_id = node_id,
        quote_id = %quote_id,
        amount = %amount,
        unit = %unit
    );

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
    .map_err(|e| Error::RedeemMintQuote(quote_id.clone(), e))?;

    event!(name: "mint_quote_redeemed_successfully", Level::INFO,
        node_id = node_id,
        quote_id = %quote_id,
        amount = %amount,
        unit = %unit
    );

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
        .quote_event_sender
        .send(QuoteHandlerEvent::Mint(MintQuoteAction::Done {
            node_id,
            quote_id: quote_id.clone(),
        }))
        .await
        .map_err(|_| Error::SendMessage)?;

    emit_mint_quote_redeemed_event(&app, QuoteIdentifier { node_id, quote_id })
        .map_err(CommonError::EmitTauriEvent)?;

    Ok(())
}

pub async fn sync_mint_quote_until_is_paid(
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
                event!(name: "mint-quote-paid", Level::INFO, quote_id = %quote_id, "Mint quote paid");
                emit_mint_quote_paid_event(
                    &app,
                    QuoteIdentifier {
                        node_id,
                        quote_id: quote_id.clone(),
                    },
                )
                .map_err(CommonError::EmitTauriEvent)?;

                state
                    .quote_event_sender
                    .send(QuoteHandlerEvent::Mint(MintQuoteAction::Redeem {
                        node_id,
                        quote_id: quote_id.clone(),
                    }))
                    .await
                    .map_err(|_| Error::SendMessage)?;

                break;
            }
            Ok(Some(MintQuoteState::Issued)) => {
                event!(name: "mint-quote-issued", Level::INFO, quote_id = %quote_id, "Mint quote issued");
                error!(
                    "mint quote {} has been issued before it was synced as paid",
                    quote_id
                );
                break;
            }
            Ok(None) => {
                event!(name: "mint-quote-expired", Level::INFO, quote_id = %quote_id, "Mint quote expired");
                emit_remove_mint_quote_event(&app, QuoteIdentifier { node_id, quote_id })
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
    event!(name: "mint-quote-sync-finished", Level::INFO,
        "Exiting the mint quote sync loop"
    );

    Ok(())
}

#[tracing::instrument(skip(app))]
pub async fn try_pay_melt_quote(
    app: AppHandle,
    node_id: u32,
    quote_id: String,
) -> Result<(), Error> {
    let state = app.state::<AppState>();

    let MeltQuote {
        node_id,
        method,
        amount,
        unit,
        state: quote_state,
        ..
    } = {
        let pool = state.pool.get().map_err(CommonError::DbPool)?;
        match wallet::db::melt_quote::get(&pool, node_id, &quote_id).map_err(CommonError::Db)? {
            Some(mq) => mq,
            None => return Err(CommonError::QuoteNotFound(quote_id.clone()).into()),
        }
    };

    if quote_state != MeltQuoteState::Unpaid {
        event!(name: "cannot_pay_non_unpaid_melt_quote", Level::WARN,
            node_id = node_id,
            quote_id = %quote_id,
            quote_state = ?quote_state
        );
        return Err(Error::InvalidMeltQuoteState {
            expected: MeltQuoteState::Paid,
            got: quote_state,
        });
    }

    let mut node_client = state
        .get_node_client_connection(node_id)
        .await
        .map_err(CommonError::CachedConnection)?;

    event!(name: "paying_melt_quote_with_node", Level::INFO,
        node_id = node_id,
        quote_id = %quote_id,
        amount = %amount,
        unit = %unit,
        method = %method
    );

    let melt_response = match wallet::melt::pay_quote(
        crate::SEED_PHRASE_MANAGER,
        state.pool.clone(),
        &mut node_client,
        node_id,
        quote_id.clone(),
        amount,
        method,
        &unit,
    )
    .await
    {
        Ok(res) => res,
        Err(e) => return Err(Error::PayMeltQuote(quote_id, e)),
    };

    event!(name: "melt_quote_paid_successfully", Level::INFO,
        node_id = node_id,
        quote_id = %quote_id,
        response_state = melt_response.state
    );

    emit_balance_decrease_event(
        &app,
        BalanceChange {
            node_id,
            unit,
            amount: amount.into(),
        },
    )
    .map_err(CommonError::EmitTauriEvent)?;

    if melt_response.state == node_client::MeltQuoteState::MlqsPending as i32 {
        state
            .quote_event_sender
            .send(QuoteHandlerEvent::Melt(
                MeltQuoteAction::WaitOnChainPayment {
                    node_id,
                    quote_id: quote_id.clone(),
                },
            ))
            .await
            .map_err(|_| CommonError::QuoteHandlerChannel)?;
    } else if melt_response.state == node_client::MeltQuoteState::MlqsPaid as i32 {
        state
            .quote_event_sender
            .send(QuoteHandlerEvent::Melt(MeltQuoteAction::Done {
                node_id,
                quote_id: quote_id.clone(),
            }))
            .await
            .map_err(|_| CommonError::QuoteHandlerChannel)?;
    } else {
        // Should not occur
        return Err(Error::InvalidMeltQuoteState {
            expected: MeltQuoteState::Pending,
            got: MeltQuoteState::Unpaid,
        });
    }

    emit_melt_quote_paid_event(&app, QuoteIdentifier { node_id, quote_id })
        .map_err(CommonError::EmitTauriEvent)?;

    Ok(())
}

#[tracing::instrument(skip(app))]
pub async fn sync_melt_quote_until_is_paid(
    app: AppHandle,
    node_id: u32,
    quote_id: String,
) -> Result<(), Error> {
    let state = app.state::<AppState>();
    let melt_quote = {
        let pool = state.pool.get().map_err(CommonError::DbPool)?;
        match wallet::db::melt_quote::get(&pool, node_id, &quote_id).map_err(CommonError::Db)? {
            Some(mq) => mq,
            None => return Err(CommonError::QuoteNotFound(quote_id.clone()).into()),
        }
    };
    let mut node_client = state
        .get_node_client_connection(node_id)
        .await
        .map_err(CommonError::CachedConnection)?;

    event!(name: "waiting_for_melt_quote_payment", Level::INFO,
        node_id = node_id,
        quote_id = %quote_id
    );

    match wallet::melt::wait_for_payment(
        state.pool.clone(),
        &mut node_client,
        melt_quote.method,
        quote_id.clone(),
    )
    .await
    .map_err(Error::WaitForMeltQuotePayment)?
    {
        Some(_tx_id) => {
            event!(name: "melt_quote_payment_confirmed", Level::INFO,
                node_id = node_id,
                quote_id = %quote_id
            );
            emit_melt_quote_redeemed_event(
                &app,
                QuoteIdentifier {
                    node_id,
                    quote_id: quote_id.clone(),
                },
            )
            .map_err(CommonError::EmitTauriEvent)?;

            state
                .quote_event_sender
                .send(QuoteHandlerEvent::Melt(MeltQuoteAction::Done {
                    node_id,
                    quote_id,
                }))
                .await
                .map_err(|_| CommonError::QuoteHandlerChannel)?;
        }
        None => {
            // This is not supposed to happen.
            // If we were able to pay it, it cannot expire anymore
            // But let's handle it anyway, at least for the front
            emit_remove_melt_quote_event(
                &app,
                QuoteIdentifier {
                    node_id,
                    quote_id: quote_id.clone(),
                },
            )
            .map_err(CommonError::EmitTauriEvent)?;
            state
                .quote_event_sender
                .send(QuoteHandlerEvent::Melt(MeltQuoteAction::Done {
                    node_id,
                    quote_id,
                }))
                .await
                .map_err(|_| CommonError::QuoteHandlerChannel)?;
        }
    }

    Ok(())
}
