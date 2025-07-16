use rusqlite::{Connection, Result};
use thiserror::Error;

#[derive(Debug)]
pub struct PaymentEvent {
    pub index: u64,
    pub asset: String,
    pub payee: String,
    pub invoice_id: String,
    pub payer: String,
    pub amount_low: String,
    pub amount_high: String,
}

#[derive(Debug)]
pub struct Block {
    pub id: String,
    pub number: u64,
}

impl From<&apibara_core::starknet::v1alpha2::BlockHeader> for Block {
    fn from(value: &apibara_core::starknet::v1alpha2::BlockHeader) -> Self {
        Self {
            id: value.block_hash.as_ref().unwrap().to_string(),
            number: value.block_number,
        }
    }
}

#[derive(Debug, Error)]
pub enum TryPaymentEventFromApibaraEvent {
    #[error("event has no key at index {0}")]
    Key(u8),
    #[error("event has no data at index {0}")]
    Data(u8),
}

impl TryFrom<&apibara_core::starknet::v1alpha2::Event> for PaymentEvent {
    type Error = TryPaymentEventFromApibaraEvent;

    fn try_from(value: &apibara_core::starknet::v1alpha2::Event) -> Result<Self, Self::Error> {
        Ok(Self {
            index: value.index,
            asset: value
                .keys
                .get(1)
                .ok_or(TryPaymentEventFromApibaraEvent::Key(1))?
                .to_string(),
            payee: value
                .keys
                .get(2)
                .ok_or(TryPaymentEventFromApibaraEvent::Key(2))?
                .to_string(),
            #[allow(clippy::get_first)]
            invoice_id: value
                .data
                .get(0)
                .ok_or(TryPaymentEventFromApibaraEvent::Data(0))?
                .to_string(),
            payer: value
                .data
                .get(1)
                .ok_or(TryPaymentEventFromApibaraEvent::Data(2))?
                .to_string(),
            amount_low: value
                .data
                .get(2)
                .ok_or(TryPaymentEventFromApibaraEvent::Data(3))?
                .to_string(),
            amount_high: value
                .data
                .get(3)
                .ok_or(TryPaymentEventFromApibaraEvent::Data(4))?
                .to_string(),
        })
    }
}

pub fn create_tables(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;

    const CREATE_TABLE_BLOCK: &str = r#"
        CREATE TABLE IF NOT EXISTS block (
            id TEXT PRIMARY KEY,
            number INTEGER NOT NULL
        )"#;

    const CREATE_TABLE_PAYMENT_EVENT: &str = r#"
        CREATE TABLE IF NOT EXISTS payment_event (
            id INTEGER PRIMARY KEY,
            block_id TEXT NOT NULL REFERENCES block(id) ON DELETE CASCADE,
            tx_hash TEXT NOT NULL,
            event_index INTEGER NOT NULL,
            payee TEXT NOT NULL,
            asset TEXT NOT NULL,
            invoice_id TEXT NOT NULL,
            payer TEXT NOT NULL,
            amount_low TEXT NOT NULL,
            amount_high TEXT NOT NULL
        )"#;

    tx.execute(CREATE_TABLE_BLOCK, ())?;
    tx.execute(CREATE_TABLE_PAYMENT_EVENT, ())?;

    tx.commit()?;

    Ok(())
}

pub fn insert_new_block(conn: &Connection, block: &Block) -> Result<()> {
    const INSERT_NEW_BLOCK: &str = r#"INSERT INTO block (id, number) VALUES ($1, $2)"#;

    conn.execute(INSERT_NEW_BLOCK, (&block.id, block.number))?;

    Ok(())
}

pub fn insert_payment_event(
    conn: &Connection,
    block_id: &str,
    transaction_hash: &str,
    payment_event: &PaymentEvent,
) -> Result<()> {
    const INSERT_PAYMENT_EVENT: &str = r#"
        INSERT INTO payment_event
            (block_id, tx_hash, event_index, payee, asset, invoice_id, payer, amount_low, amount_high)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#;

    conn.execute(
        INSERT_PAYMENT_EVENT,
        (
            block_id,
            transaction_hash,
            &payment_event.index,
            &payment_event.payee,
            &payment_event.asset,
            &payment_event.invoice_id,
            &payment_event.payer,
            &payment_event.amount_low,
            &payment_event.amount_high,
        ),
    )?;

    Ok(())
}

pub fn invalidate(conn: &Connection, height: u64) -> Result<()> {
    const INVALIDATE: &str = r#"DELETE * FROM payent_event WHERE number > $1"#;

    conn.execute(INVALIDATE, (height,))?;

    Ok(())
}
