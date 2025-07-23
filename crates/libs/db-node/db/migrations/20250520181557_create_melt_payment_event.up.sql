ALTER TABLE payment_event RENAME TO mint_payment_event;

ALTER TABLE melt_quote ADD CONSTRAINT melt_quote_invoice_id_unique UNIQUE (invoice_id);

CREATE TABLE IF NOT EXISTS substreams_cursor (
    name TEXT PRIMARY KEY,
    cursor TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS substreams_starknet_block (
    id TEXT PRIMARY KEY,
    number BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS starknet_block_number ON substreams_starknet_block(number);

CREATE TABLE IF NOT EXISTS melt_payment_event (
    block_id TEXT NOT NULL REFERENCES substreams_starknet_block(id) ON DELETE CASCADE,
    tx_hash TEXT NOT NULL,
    event_index BIGINT NOT NULL,
    payee TEXT NOT NULL,
    asset TEXT NOT NULL,
    invoice_id BYTEA NOT NULL REFERENCES melt_quote(invoice_id),
    payer TEXT NOT NULL,
    amount_low TEXT NOT NULL,
    amount_high TEXT NOT NULL,
    PRIMARY KEY (tx_hash, event_index)
);

ALTER TABLE melt_quote DROP COLUMN transfer_id;

