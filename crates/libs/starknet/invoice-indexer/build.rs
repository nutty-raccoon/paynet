use cainome::rs::Abigen;
use std::collections::HashMap;

fn main() {
    // Aliases added from the ABI
    let aliases = HashMap::new();

    let invoice_contract_abigen = Abigen::new(
        "invoice_contract",
        "./abi/invoice_contract_contract.abi.json",
    )
    .with_types_aliases(aliases)
    .with_derives(vec![
        "serde::Serialize".to_string(),
        "serde::Deserialize".to_string(),
    ]);

    invoice_contract_abigen
        .generate()
        .expect("Fail to generate bindings")
        .write_to_file("./src/abi/invoice_contract_contract.rs")
        .unwrap();
}
