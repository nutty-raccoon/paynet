use cainome::rs::Abigen;
use std::collections::HashMap;

fn main() {
    // Aliases added from the ABI
    let mut aliases = HashMap::new();
    aliases.insert(
        String::from("openzeppelin_account::account::AccountComponent::Event"),
        String::from("AccountComponentEvent"),
    );
    aliases.insert(
        String::from("openzeppelin_introspection::src5::SRC5Component::Event"),
        String::from("SRC5ComponentEvent"),
    );
    aliases.insert(
        String::from("openzeppelin_upgrades::upgradeable::UpgradeableComponent::Event"),
        String::from("UpgradeableComponentEvent"),
    );

    let invoice_abigen =
        Abigen::new("invoice", "./abi/invoice_contract.abi.json").with_types_aliases(aliases).with_derives(vec!["serde::Serialize".to_string(), "serde::Deserialize".to_string()]);

        invoice_abigen
            .generate()
            .expect("Fail to generate bindings")
            .write_to_file("./src/abi/invoice_contract.rs")
            .unwrap();
}