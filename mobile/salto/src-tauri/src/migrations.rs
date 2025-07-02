use tauri_plugin_sql::{Migration, MigrationKind};

pub fn migrations() -> Vec<Migration> {
    vec![
        // Define your migrations here
        Migration {
            version: 1,
            description: "create_table_node",
            sql: wallet::db::node::CREATE_TABLE_NODE,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "create_table_keyset",
            sql: wallet::db::CREATE_TABLE_KEYSET,
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
            sql: wallet::db::CREATE_TABLE_MINT_QUOTE,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 5,
            description: "create_table_melt_quote",
            sql: wallet::db::CREATE_TABLE_MELT_QUOTE,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 6,
            description: "create_table_proof",
            sql: wallet::db::proof::CREATE_TABLE_PROOF,
            kind: MigrationKind::Up,
        },
    ]
}
