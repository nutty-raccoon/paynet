use anyhow::{Context, Error, format_err};
use prost::Message;

use crate::pb::sf::substreams::v1::{
    Package,
    module::input::{Input, Params},
};

#[derive(Debug, Clone)]
pub struct Param {
    pub module_name: String,
    pub expression: String,
}

pub fn read_package(path: &str, params: Vec<Param>) -> Result<Package, Error> {
    let content = std::fs::read(path)
        .context(format_err!("read package from file '{}'", path.to_string()))?;
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
