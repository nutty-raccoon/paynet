ALTER TABLE mint_quote ADD COLUMN pubkey BYTEA CHECK (length(pubkey) = 33);
