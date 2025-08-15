use std::sync::LazyLock;

use starknet_core::types::contract::AbiEntry;

pub static INVOICE_CONTRACT_ABI: LazyLock<Vec<AbiEntry>> = LazyLock::new(|| {
    serde_json::from_str(INVOICE_CONTRACT_ABI_STRING)
        .expect("Failed to deserialize invoice contract ABI")
});

pub static IERC20_CONTRACT_ABI: LazyLock<AbiEntry> = LazyLock::new(|| {
    serde_json::from_str(IERC20_CONTRACT_ABI_STRING)
        .expect("Failed to deserialize IERC20 contract ABI")
});

const INVOICE_CONTRACT_ABI_STRING: &str = r#"[{"type":"impl","name":"InvoicePaymentImpl","interface_name":"invoice_payment::IInvoicePayment"},{"type":"struct","name":"core::integer::u256","members":[{"name":"low","type":"core::integer::u128"},{"name":"high","type":"core::integer::u128"}]},{"type":"interface","name":"invoice_payment::IInvoicePayment","items":[{"type":"function","name":"pay_invoice","inputs":[{"name":"quote_id_hash","type":"core::felt252"},{"name":"expiry","type":"core::integer::u64"},{"name":"asset","type":"core::starknet::contract_address::ContractAddress"},{"name":"amount","type":"core::integer::u256"},{"name":"payee","type":"core::starknet::contract_address::ContractAddress"}],"outputs":[],"state_mutability":"external"}]},{"type":"event","name":"invoice_payment::InvoicePayment::Remittance","kind":"struct","members":[{"name":"asset","type":"core::starknet::contract_address::ContractAddress","kind":"key"},{"name":"payee","type":"core::starknet::contract_address::ContractAddress","kind":"key"},{"name":"invoice_id","type":"core::felt252","kind":"data"},{"name":"payer","type":"core::starknet::contract_address::ContractAddress","kind":"data"},{"name":"amount","type":"core::integer::u256","kind":"data"}]},{"type":"event","name":"invoice_payment::InvoicePayment::Event","kind":"enum","variants":[{"name":"Remittance","type":"invoice_payment::InvoicePayment::Remittance","kind":"nested"}]}]"#;
const IERC20_CONTRACT_ABI_STRING: &str = r#"{
    "name": "openzeppelin::token::erc20::interface::IERC20",
    "type": "interface",
    "items": [
      {
        "name": "name",
        "type": "function",
        "inputs": [],
        "outputs": [
          {
            "type": "core::felt252"
          }
        ],
        "state_mutability": "view"
      },
      {
        "name": "symbol",
        "type": "function",
        "inputs": [],
        "outputs": [
          {
            "type": "core::felt252"
          }
        ],
        "state_mutability": "view"
      },
      {
        "name": "decimals",
        "type": "function",
        "inputs": [],
        "outputs": [
          {
            "type": "core::integer::u8"
          }
        ],
        "state_mutability": "view"
      },
      {
        "name": "total_supply",
        "type": "function",
        "inputs": [],
        "outputs": [
          {
            "type": "core::integer::u256"
          }
        ],
        "state_mutability": "view"
      },
      {
        "name": "balance_of",
        "type": "function",
        "inputs": [
          {
            "name": "account",
            "type": "core::starknet::contract_address::ContractAddress"
          }
        ],
        "outputs": [
          {
            "type": "core::integer::u256"
          }
        ],
        "state_mutability": "view"
      },
      {
        "name": "allowance",
        "type": "function",
        "inputs": [
          {
            "name": "owner",
            "type": "core::starknet::contract_address::ContractAddress"
          },
          {
            "name": "spender",
            "type": "core::starknet::contract_address::ContractAddress"
          }
        ],
        "outputs": [
          {
            "type": "core::integer::u256"
          }
        ],
        "state_mutability": "view"
      },
      {
        "name": "transfer",
        "type": "function",
        "inputs": [
          {
            "name": "recipient",
            "type": "core::starknet::contract_address::ContractAddress"
          },
          {
            "name": "amount",
            "type": "core::integer::u256"
          }
        ],
        "outputs": [
          {
            "type": "core::bool"
          }
        ],
        "state_mutability": "external"
      },
      {
        "name": "transfer_from",
        "type": "function",
        "inputs": [
          {
            "name": "sender",
            "type": "core::starknet::contract_address::ContractAddress"
          },
          {
            "name": "recipient",
            "type": "core::starknet::contract_address::ContractAddress"
          },
          {
            "name": "amount",
            "type": "core::integer::u256"
          }
        ],
        "outputs": [
          {
            "type": "core::bool"
          }
        ],
        "state_mutability": "external"
      },
      {
        "name": "approve",
        "type": "function",
        "inputs": [
          {
            "name": "spender",
            "type": "core::starknet::contract_address::ContractAddress"
          },
          {
            "name": "amount",
            "type": "core::integer::u256"
          }
        ],
        "outputs": [
          {
            "type": "core::bool"
          }
        ],
        "state_mutability": "external"
      }
    ]
  }"#;
