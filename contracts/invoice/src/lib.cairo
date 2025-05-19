//! An erc20 transfer with richer event
//!
//! The sole purpose of this contract is to provide the ability to pass a transfer reference
//! in a way similar to [eip-7699](https://github.com/ethereum/ERCs/blob/master/ERCS/erc-7699.md).
//! 
//! We use it during the mint process:
//! 1. the user require a mint quote form the node, it comes with an UUID.
//! 2. the user deposit to the node address using this Invoice contract,
//     providing a felt build using `from_bytes_be` on the hash of this UUID as `invoice_id`
//! 3. the node listen to on-chain deposit to its address, and use the `invoice_id` to flag the correct quote as `PAID`
//! 4. the user call the node's `mint` route with the original UUID and receive the corresponding amount of tokens

use starknet::ContractAddress;

#[starknet::interface]
pub trait IInvoicePayment<TContractState> {
    /// Execute an erc20 transfer and emit the rich event 
    fn pay_invoice(
        ref self: TContractState,
        quote_id_hash: felt252,
        expiry: u64,
        asset: ContractAddress,
        amount: u256,
        payee: ContractAddress,
    );
}


#[starknet::contract]
pub mod InvoicePayment {
    use starknet::{get_caller_address, ContractAddress};
    use openzeppelin_token::erc20::interface::{IERC20Dispatcher, IERC20DispatcherTrait};
    use core::poseidon::PoseidonTrait;
    use core::hash::HashStateTrait;

    #[storage]
    struct Storage {}

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        Remittance: Remittance,
    }

    /// A deposit was made for `invoice_id`
    #[derive(Debug, Drop, starknet::Event)]
    pub struct Remittance {
        // Keys
        #[key]
        pub payee: ContractAddress,
        #[key]
        pub asset: ContractAddress,
        // Data
        pub invoice_id: felt252,
        pub payer: ContractAddress,
        pub amount: u256,
    }

    #[abi(embed_v0)]
    impl InvoicePaymentImpl of super::IInvoicePayment<ContractState> {
        fn pay_invoice(
            ref self: ContractState,
            quote_id_hash: felt252,
            expiry: u64,
            asset: ContractAddress,
            amount: u256,
            payee: ContractAddress,
        ) {
            let payer = get_caller_address();
            let erc20_dispatcher = IERC20Dispatcher { contract_address: asset };

            assert!(expiry <= starknet::get_block_timestamp());

            let mut hash_state = PoseidonTrait::new();
            hash_state.update(quote_id_hash);
            hash_state.update(expiry.into());
            let invoice_id = hash_state.finalize();

            assert!(erc20_dispatcher.transfer_from(payer, payee, amount));

            self.emit(Remittance { payee, asset, invoice_id, payer, amount });
        }
    }
}
