use openzeppelin_presets::interfaces::ERC20UpgradeableABIDispatcherTrait;

use snforge_std::{
    ContractClassTrait, DeclareResultTrait, CheatSpan, EventSpyAssertionsTrait, start_cheat_block_timestamp_global
};
use core::starknet::{ContractAddress, contract_address_const};

use invoice_payment::{IInvoicePaymentDispatcherTrait, IInvoicePaymentDispatcher};
use openzeppelin_presets::interfaces::{ERC20UpgradeableABIDispatcher};
use openzeppelin_utils::serde::SerializedAppend;
use openzeppelin_token::erc20::ERC20Component;
use core::poseidon::poseidon_hash_span;

pub const SUPPLY: u256 = 1_000_000_000_000_000_000; // 1e18

pub fn NAME() -> ByteArray {
    "NAME"
}

pub fn SYMBOL() -> ByteArray {
    "SYMBOL"
}

pub fn OWNER() -> ContractAddress {
    contract_address_const::<'owner'>()
}

pub fn SENDER() -> ContractAddress {
    contract_address_const::<'sender'>()
}

pub fn RECIPIENT() -> ContractAddress {
    contract_address_const::<'recipient'>()
}

pub fn setup_erc20(recipient: ContractAddress) -> ERC20UpgradeableABIDispatcher {
    let mut calldata = array![];

    calldata.append_serde(NAME());
    calldata.append_serde(SYMBOL());
    calldata.append_serde(SUPPLY);
    calldata.append_serde(recipient);
    calldata.append_serde(recipient);

    let contract = snforge_std::declare("ERC20Upgradeable").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@calldata).unwrap();

    ERC20UpgradeableABIDispatcher { contract_address: contract_address }
}

pub fn setup_invoice_payment() -> IInvoicePaymentDispatcher {
    let contract = snforge_std::declare("InvoicePayment").unwrap().contract_class();

    let (contract_address, _) = contract.deploy(@array![]).unwrap();

    IInvoicePaymentDispatcher { contract_address: contract_address }
}


#[test]
fn it_works() {
    const AMOUNT: u256 = 200;
    let erc20_abi = setup_erc20(OWNER());
    let invoice_payment_abi = setup_invoice_payment();

    let quote_id_hash: felt252 = snforge_std::generate_random_felt();

    let mut spy = snforge_std::spy_events();

    let expiry = 0;

    // OWNER fund SENDER
    snforge_std::cheat_caller_address(
        erc20_abi.contract_address, OWNER(), CheatSpan::TargetCalls(1),
    );
    erc20_abi.transfer(SENDER(), AMOUNT);

    // SENDER allow invoice_payment
    snforge_std::cheat_caller_address(
        erc20_abi.contract_address, SENDER(), CheatSpan::TargetCalls(1),
    );
    erc20_abi.approve(invoice_payment_abi.contract_address, AMOUNT);

    // SENDER pay invoice
    snforge_std::cheat_caller_address(
        invoice_payment_abi.contract_address, SENDER(), CheatSpan::TargetCalls(1),
    );

    invoice_payment_abi.pay_invoice(quote_id_hash, expiry, erc20_abi.contract_address, AMOUNT, RECIPIENT());

    let span = [quote_id_hash, expiry.into()].span();
    let id_hash = poseidon_hash_span(span);

    // Payment went through
    assert_eq!(erc20_abi.balance_of(SENDER()), 0);
    assert_eq!(erc20_abi.balance_of(RECIPIENT()), AMOUNT);
    assert_eq!(erc20_abi.balance_of(OWNER()), SUPPLY - AMOUNT);

    // Event were emitted
    spy
        .assert_emitted(
            @array![
                (
                    erc20_abi.contract_address,
                    ERC20Component::Event::Transfer(
                        ERC20Component::Transfer { from: SENDER(), to: RECIPIENT(), value: AMOUNT },
                    ),
                ),
            ],
        );

    spy
        .assert_emitted(
            @array![
                (
                    invoice_payment_abi.contract_address,
                    invoice_payment::InvoicePayment::Event::Remittance(
                        invoice_payment::InvoicePayment::Remittance {
                            payee: RECIPIENT(),
                            asset: erc20_abi.contract_address,
                            invoice_id: id_hash,
                            amount: AMOUNT,
                            payer: SENDER(),
                        },
                    ),
                ),
            ],
        );
}


#[test]
fn good_invoice_id() {
    const AMOUNT: u256 = 200;
    let erc20_abi = setup_erc20(OWNER());
    let invoice_payment_abi = setup_invoice_payment();

    let quote_id_hash: felt252 = 5;

    let mut spy = snforge_std::spy_events();

    let expiry = 0;

    // OWNER fund SENDER
    snforge_std::cheat_caller_address(
        erc20_abi.contract_address, OWNER(), CheatSpan::TargetCalls(1),
    );
    erc20_abi.transfer(SENDER(), AMOUNT);

    // SENDER allow invoice_payment
    snforge_std::cheat_caller_address(
        erc20_abi.contract_address, SENDER(), CheatSpan::TargetCalls(1),
    );
    erc20_abi.approve(invoice_payment_abi.contract_address, AMOUNT);

    // SENDER pay invoice
    snforge_std::cheat_caller_address(
        invoice_payment_abi.contract_address, SENDER(), CheatSpan::TargetCalls(1),
    );

    invoice_payment_abi.pay_invoice(quote_id_hash, expiry, erc20_abi.contract_address, AMOUNT, RECIPIENT());

    // Id hash generated by the poseidon function in rust
    let id_hash: felt252 = 593577557340018314678415990879734807818083626248355598150054109794228418409;
    spy
        .assert_emitted(
            @array![
                (
                    invoice_payment_abi.contract_address,
                    invoice_payment::InvoicePayment::Event::Remittance(
                        invoice_payment::InvoicePayment::Remittance {
                            payee: RECIPIENT(),
                            asset: erc20_abi.contract_address,
                            invoice_id: id_hash,
                            amount: AMOUNT,
                            payer: SENDER(),
                        },
                    ),
                ),
            ],
        );
}

#[test]
#[should_panic]
fn bad_expiry() {
    const AMOUNT: u256 = 200;
    let erc20_abi = setup_erc20(OWNER());
    let invoice_payment_abi = setup_invoice_payment();

    let quote_id_hash: felt252 = 5;

    let mut spy = snforge_std::spy_events();

    let expiry = 1;
    start_cheat_block_timestamp_global(3);

    // OWNER fund SENDER
    snforge_std::cheat_caller_address(
        erc20_abi.contract_address, OWNER(), CheatSpan::TargetCalls(1),
    );
    erc20_abi.transfer(SENDER(), AMOUNT);

    // SENDER allow invoice_payment
    snforge_std::cheat_caller_address(
        erc20_abi.contract_address, SENDER(), CheatSpan::TargetCalls(1),
    );
    erc20_abi.approve(invoice_payment_abi.contract_address, AMOUNT);

    // SENDER pay invoice
    snforge_std::cheat_caller_address(
        invoice_payment_abi.contract_address, SENDER(), CheatSpan::TargetCalls(1),
    );

    invoice_payment_abi.pay_invoice(quote_id_hash, expiry, erc20_abi.contract_address, AMOUNT, RECIPIENT());

    // Id hash generated by the poseidon function in rust
    let id_hash: felt252 = 1278717218625069357298452719407742240477041713675101626249510460722529827614;
    spy
        .assert_emitted(
            @array![
                (
                    invoice_payment_abi.contract_address,
                    invoice_payment::InvoicePayment::Event::Remittance(
                        invoice_payment::InvoicePayment::Remittance {
                            payee: RECIPIENT(),
                            asset: erc20_abi.contract_address,
                            invoice_id: id_hash,
                            amount: AMOUNT,
                            payer: SENDER(),
                        },
                    ),
                ),
            ],
        );
}