use anyhow::{Context, Error, format_err};
use prost::Message;

use crate::pb::sf::substreams::v1::{
    Package,
    module::input::{Input, Params},
};

struct Param {
    pub module_name: String,
    pub expression: String,
}

fn read_package(input: &str, params: Vec<Param>) -> Result<Package, Error> {
    let content =
        std::fs::read(input).context(format_err!("read package from file '{}'", input))?;
    let mut package = Package::decode(content.as_ref()).context("decode command")?;

    if !params.is_empty() {
        // Find the module by name and apply the block filter
        if let Some(modules) = &mut package.modules {
            for param in params {
                if let Some(module) = modules
                    .modules
                    .iter_mut()
                    .find(|m| m.name == param.module_name)
                {
                    module.inputs[0].input = Some(Input::Params(Params {
                        value: param.expression,
                    }));
                }
            }
        }
        Ok(package)
    } else {
        Ok(package)
    }
}
