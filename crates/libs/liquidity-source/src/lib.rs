mod deposit;
use std::fmt::{LowerHex, UpperHex};

pub use deposit::DepositInterface;
mod withdraw;
use nuts::traits::Unit;
use uuid::Uuid;
pub use withdraw::WithdrawInterface;

pub trait LiquiditySource {
    type InvoiceId: Into<[u8; 32]> + LowerHex + UpperHex + Clone + Send + Sync + 'static;
    type Unit: Unit;
    type Depositer: DepositInterface<InvoiceId = Self::InvoiceId>;
    type Withdrawer: WithdrawInterface<InvoiceId = Self::InvoiceId, Unit = Self::Unit>;

    fn depositer(&self) -> Self::Depositer;
    fn withdrawer(&self) -> Self::Withdrawer;
    fn compute_invoice_id(&self, quote_id: Uuid, expiry: u64) -> Self::InvoiceId;
}
