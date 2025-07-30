mod add_node;
mod deposit;
mod get_nodes_balance;
mod wad;

pub use add_node::add_node;
pub use deposit::{create_mint_quote, redeem_quote};
pub use get_nodes_balance::get_nodes_balance;
pub use wad::{create_wads, get_wad_history, receive_wads, sync_wads};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceChange {
    node_id: u32,
    unit: String,
    amount: u64,
}
