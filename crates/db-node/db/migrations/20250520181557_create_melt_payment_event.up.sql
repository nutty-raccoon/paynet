ALTER TABLE payment_event RENAME TO mint_payment_event;

ALTER TABLE melt_quote ADD CONSTRAINT melt_quote_invoice_id_unique UNIQUE (invoice_id);

CREATE TABLE IF NOT EXISTS melt_payment_event (
    block_id TEXT NOT NULL,
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

