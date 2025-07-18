 CREATE TABLE IF NOT EXISTS invoice_contract_events (
            id TEXT PRIMARY KEY,
            block_id TEXT NOT NULL,
            tx_hash TEXT NOT NULL,
            event_index INTEGER NOT NULL,
            asset TEXT NOT NULL,
            payee TEXT NOT NULL,
            invoice_id TEXT NOT NULL,
            payer TEXT NOT NULL,
            amount_low TEXT NOT NULL,
            amount_high TEXT NOT NULL
        );
