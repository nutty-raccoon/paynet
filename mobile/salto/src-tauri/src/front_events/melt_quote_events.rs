use tauri::{AppHandle, Emitter, Error};

use super::{
    PendingQuoteData, QUOTE_EVENT, QuoteEventData, QuoteIdentifier, QuoteType, UnifiedQuoteEvent,
};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeltQuoteCreatedEvent {
    pub node_id: u32,
    pub melt_quote: PendingQuoteData,
}

pub fn emit_melt_quote_created_event(
    app: &AppHandle,
    event: MeltQuoteCreatedEvent,
) -> Result<(), Error> {
    let unified_event = UnifiedQuoteEvent {
        r#type: "created".to_string(),
        quote_type: QuoteType::Melt,
        node_id: event.node_id,
        data: QuoteEventData::Created {
            quote: event.melt_quote,
        },
    };
    app.emit(QUOTE_EVENT, unified_event)
}

pub fn emit_melt_quote_paid_event(app: &AppHandle, event: QuoteIdentifier) -> Result<(), Error> {
    let unified_event = UnifiedQuoteEvent {
        r#type: "paid".to_string(),
        quote_type: QuoteType::Melt,
        node_id: event.node_id,
        data: QuoteEventData::Identifier {
            quote_id: event.quote_id,
        },
    };
    app.emit(QUOTE_EVENT, unified_event)
}

pub fn emit_melt_quote_redeemed_event(
    app: &AppHandle,
    event: QuoteIdentifier,
) -> Result<(), Error> {
    let unified_event = UnifiedQuoteEvent {
        r#type: "redeemed".to_string(),
        quote_type: QuoteType::Melt,
        node_id: event.node_id,
        data: QuoteEventData::Identifier {
            quote_id: event.quote_id,
        },
    };
    app.emit(QUOTE_EVENT, unified_event)
}

pub fn emit_remove_melt_quote_event(app: &AppHandle, event: QuoteIdentifier) -> Result<(), Error> {
    let unified_event = UnifiedQuoteEvent {
        r#type: "removed".to_string(),
        quote_type: QuoteType::Melt,
        node_id: event.node_id,
        data: QuoteEventData::Identifier {
            quote_id: event.quote_id,
        },
    };
    app.emit(QUOTE_EVENT, unified_event)
}
