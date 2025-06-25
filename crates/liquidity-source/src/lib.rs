mod deposit;
use std::fmt::{LowerHex, UpperHex};

pub use deposit::DepositInterface;
mod withdraw;
use uuid::Uuid;
pub use withdraw::{WithdrawAmount, WithdrawInterface, WithdrawRequest};

// Implementations
#[cfg(feature = "mock")]
pub mod mock;

pub trait LiquiditySource {
    type InvoiceId: Into<[u8; 32]> + LowerHex + UpperHex + Clone + Send + Sync + 'static;
    type Depositer: DepositInterface<InvoiceId = Self::InvoiceId>;
    type Withdrawer: WithdrawInterface<InvoiceId = Self::InvoiceId>;

    fn depositer(&self) -> Self::Depositer;
    fn withdrawer(&self) -> Self::Withdrawer;
    fn compute_invoice_id(&self, quote_id: Uuid, expiry: u64) -> Self::InvoiceId;
}
