use tauri_plugin_sql::{Migration, MigrationKind};

pub fn migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "create_table_node",
            sql: wallet::db::node::CREATE_TABLE_NODE,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "create_table_keyset",
            sql: wallet::db::keyset::CREATE_TABLE_KEYSET,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "create_table_key",
            sql: wallet::db::CREATE_TABLE_KEY,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 4,
            description: "create_table_mint_quote",
            sql: wallet::db::mint_quote::CREATE_TABLE_MINT_QUOTE,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 5,
            description: "create_table_melt_quote",
            sql: wallet::db::melt_quote::CREATE_TABLE_MELT_QUOTE,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 6,
            description: "create_table_proof",
            sql: wallet::db::proof::CREATE_TABLE_PROOF,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 7,
            description: "create_table_wallet",
            sql: wallet::db::wallet::CREATE_TABLE_WALLET,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 8,
            description: "create_table_wad",
            sql: wallet::db::wad::CREATE_TABLE_WAD,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 9,
            description: "create_table_wad_proof",
            sql: wallet::db::wad::CREATE_TABLE_WAD_PROOF,
            kind: MigrationKind::Up,
        },
    ]
}
