// Traits
mod deposit;

pub use deposit::DepositInterface;
mod withdraw;
pub use withdraw::{WithdrawAmount, WithdrawInterface, WithdrawRequest};

// Implementations
#[cfg(feature = "mock")]
pub mod mock;

pub trait LiquiditySource {
    type Depositer: DepositInterface;
    type Withdrawer: WithdrawInterface;

    fn depositer(&self) -> Self::Depositer;
    fn withdrawer(&self) -> Self::Withdrawer;
}
