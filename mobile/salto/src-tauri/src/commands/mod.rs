mod add_node;
mod deposit;
mod get_nodes_balance;
mod get_prices;
mod wad;
mod wallet;

pub use add_node::add_node;
pub use deposit::{create_mint_quote, redeem_quote};
pub use get_nodes_balance::get_nodes_balance;
pub use get_prices::{
    get_currencies, price_provider_add_assets, price_provider_add_currencies, PriceResponce,
};
pub use wad::{create_wads, get_wad_history, receive_wads, sync_wads};

pub use wallet::{check_wallet_exists, init_wallet, restore_wallet};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceChange {
    node_id: u32,
    unit: String,
    amount: u64,
}
