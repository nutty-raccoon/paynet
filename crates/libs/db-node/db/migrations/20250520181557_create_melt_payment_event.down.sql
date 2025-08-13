DROP TABLE melt_payment_event;
ALTER TABLE melt_quote DROP CONSTRAINT melt_quote_invoice_id_unique;
ALTER TABLE mint_payment_event RENAME TO payment_event ;
ALTER TABLE melt_quote ADD COLUMN transfer_id BYTEA;

DROP TABLE substreams_cursor;
DROP TABLE substreams_starknet_block;
