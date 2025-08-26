CREATE TABLE IF NOT EXISTS substreams_eth_block (
    id TEXT PRIMARY KEY,
    number BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS eth_block_number ON substreams_eth_block(number);

CREATE TABLE IF NOT EXISTS eth_melt_payment_event (
    block_id TEXT NOT NULL REFERENCES substreams_eth_block(id) ON DELETE CASCADE,
    tx_hash TEXT NOT NULL,
    event_index BIGINT NOT NULL,
    payee TEXT NOT NULL,
    asset TEXT NOT NULL,
    invoice_id BYTEA NOT NULL REFERENCES melt_quote(invoice_id),
    payer TEXT NOT NULL,
    amount TEXT NOT NULL,
    PRIMARY KEY (tx_hash, event_index)
);