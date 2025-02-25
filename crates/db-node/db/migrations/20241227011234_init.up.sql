-- Keyset

CREATE TABLE IF NOT EXISTS keyset (
    id BIGINT PRIMARY KEY,
    unit TEXT NOT NULL,
    active BOOL NOT NULL,
    max_order INT2 NOT NULL,
    derivation_path_index INT4 NOT NULL
);

CREATE INDEX IF NOT EXISTS keyset_unit_index ON keyset(unit);
CREATE INDEX IF NOT EXISTS keyset_active_index ON keyset(active);

-- Blind Signature

CREATE TABLE blind_signature (
    y BYTEA CHECK (length(y) = 33) PRIMARY KEY,
    amount INT8 NOT NULL,
    keyset_id BIGINT REFERENCES keyset(id) NOT NULL,
    c BYTEA CHECK (length(c) = 33) NOT NULL
);

CREATE INDEX IF NOT EXISTS blind_signature_keyset_id_index ON blind_signature(keyset_id);

-- Proof 

CREATE TABLE IF NOT EXISTS proof (
    y BYTEA CHECK (length(y) = 33) PRIMARY KEY,
    amount INT8 NOT NULL,
    keyset_id BIGINT REFERENCES keyset(id) NOT NULL,
    secret TEXT NOT NULL,
    c BYTEA CHECK (length(c) = 33) NOT NULL,
    state INT2 NOT NULL
);

CREATE INDEX IF NOT EXISTS proof_state_index ON proof(state);
CREATE INDEX IF NOT EXISTS proof_secret_index ON proof(secret);

-- Mint quote

CREATE TYPE mint_quote_state AS ENUM ('UNPAID', 'PAID', 'ISSUED');

CREATE TABLE IF NOT EXISTS mint_quote (
    id UUID PRIMARY KEY,
    invoice_id TEXT NOT NULL UNIQUE,
    unit TEXT NOT NULL,
    amount INT8 NOT NULL,
    request TEXT NOT NULL,
    expiry TIMESTAMPTZ NOT NULL,
    state mint_quote_state NOT NULL
);

CREATE INDEX IF NOT EXISTS mint_quote_unit ON mint_quote(unit);
CREATE INDEX IF NOT EXISTS mint_quote_state ON mint_quote(state);
CREATE INDEX IF NOT EXISTS mint_quote_expiry ON mint_quote(expiry);

-- Melt quote

CREATE TYPE melt_quote_state AS ENUM ('UNPAID', 'PENDING', 'PAID');

CREATE TABLE IF NOT EXISTS melt_quote (
    id UUID PRIMARY KEY,
    unit TEXT NOT NULL,
    amount INT8 NOT NULL,
    fee INT8 NOT NULL,
    request TEXT NOT NULL,
    expiry TIMESTAMPTZ NOT NULL,
    state melt_quote_state NOT NULL
);

CREATE INDEX IF NOT EXISTS melt_quote_unit ON melt_quote(unit);
CREATE INDEX IF NOT EXISTS melt_quote_state ON melt_quote(state);
CREATE INDEX IF NOT EXISTS melt_quote_expiry ON melt_quote(expiry);

-- Starknet payment events

CREATE TABLE IF NOT EXISTS payment_event (
    block_id TEXT NOT NULL,
    tx_hash TEXT NOT NULL,
    event_index BIGINT NOT NULL,
    payee TEXT NOT NULL,
    asset TEXT NOT NULL,
    invoice_id TEXT NOT NULL REFERENCES mint_quote(invoice_id),
    payer TEXT NOT NULL,
    amount_low TEXT NOT NULL,
    amount_high TEXT NOT NULL,
    PRIMARY KEY (tx_hash, event_index)
);

