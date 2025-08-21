mod deposit;
mod get_nodes_balance;
mod node;
mod prices_provider;
mod wad;
mod wallet;

pub use deposit::{create_mint_quote, redeem_quote};
pub use get_nodes_balance::get_nodes_balance;
pub use node::{add_node, refresh_node_keysets};
pub use prices_provider::{PriceConfig, get_currencies, price_provider_add_currencies};
pub use wad::{create_wads, get_wad_history, receive_wads, sync_wads};

pub use wallet::{check_wallet_exists, init_wallet, restore_wallet};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceChange {
    node_id: u32,
    unit: String,
    amount: u64,
}
