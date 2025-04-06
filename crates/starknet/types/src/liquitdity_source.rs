impl WithdrawRequest for MeltPaymentRequest {
    fn asset(&self) -> Asset {
        self.asset
    }
}

impl WithdrawAmount for StarknetU256 {
    fn convert_from(unit: Unit, amount: nuts::Amount) -> Self {
        unit.convert_amount_into_u256(amount)
    }
}
