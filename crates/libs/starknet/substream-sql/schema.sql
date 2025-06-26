-- Required internal tables for substreams-sink-sql
CREATE TABLE IF NOT EXISTS "_blocks_" (
    id BIGINT PRIMARY KEY,
    hash TEXT NOT NULL,
    number BIGINT NOT NULL,
    parent_hash TEXT,
    timestamp TIMESTAMP
);

CREATE TABLE IF NOT EXISTS "keysentry" (
    id TEXT PRIMARY KEY,
    key TEXT NOT NULL,
    value TEXT
);

CREATE TABLE IF NOT EXISTS "tablechange" (
    id BIGSERIAL PRIMARY KEY,
    table_name TEXT NOT NULL,
    operation TEXT NOT NULL,
    block_num BIGINT NOT NULL,
    ordinal BIGINT NOT NULL
);

-- Your custom tables
CREATE TABLE IF NOT EXISTS "block" (
    id TEXT PRIMARY KEY,
    number INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS "payment_event" (
    id TEXT PRIMARY KEY,
    event_index BIGINT NOT NULL,
    payee TEXT NOT NULL,
    asset TEXT NOT NULL,
    invoice_id TEXT NOT NULL,
    payer TEXT NOT NULL,
    amount_low TEXT NOT NULL,
    amount_high TEXT NOT NULL
);