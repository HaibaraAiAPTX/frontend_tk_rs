#![deny(clippy::all)]

use std::{env::current_dir, path::Path, str::FromStr};

use aptx_frontend_tk_binding_plugin::command::CommandDescriptor;
use bootstrap::init_command_factory;
use built_in::register_built_in_command;
use napi::Error;
use swagger_tk::model::OpenAPIObject;

mod bootstrap;
mod built_in;

#[macro_use]
extern crate napi_derive;

#[napi(object)]
#[derive(Debug)]
pub struct RunCliOptions {
  pub input: String,
  pub command: String,
  pub plugin: Option<Vec<String>>,
  pub options: Vec<String>,
}

#[napi(object)]
#[derive(Debug)]
pub struct GetHelpTreeOptions {
  pub plugin: Option<Vec<String>>,
}

#[napi(object)]
#[derive(Debug)]
pub struct HelpOptionDescriptor {
  pub long: String,
  pub short: Option<String>,
  pub value_name: Option<String>,
  pub required: bool,
  pub multiple: bool,
  pub default_value: Option<String>,
  pub description: String,
}

#[napi(object)]
#[derive(Debug)]
pub struct HelpCommandDescriptor {
  pub name: String,
  pub summary: String,
  pub description: Option<String>,
  pub aliases: Vec<String>,
  pub options: Vec<HelpOptionDescriptor>,
  pub examples: Vec<String>,
  pub plugin_name: Option<String>,
  pub plugin_version: Option<String>,
}

fn to_help_descriptor(descriptor: CommandDescriptor) -> HelpCommandDescriptor {
  HelpCommandDescriptor {
    name: descriptor.name,
    summary: descriptor.summary,
    description: descriptor.description,
    aliases: descriptor.aliases,
    options: descriptor
      .options
      .into_iter()
      .map(|o| HelpOptionDescriptor {
        long: o.long,
        short: o.short.map(|c| c.to_string()),
        value_name: o.value_name,
        required: o.required,
        multiple: o.multiple,
        default_value: o.default_value,
        description: o.description,
      })
      .collect(),
    examples: descriptor.examples,
    plugin_name: descriptor.plugin_name,
    plugin_version: descriptor.plugin_version,
  }
}

#[napi]
pub fn run_cli(options: RunCliOptions) -> napi::Result<()> {
  let input = {
    let path = Path::new(&options.input);
    if path.is_absolute() {
      path.to_path_buf()
    } else {
      current_dir().unwrap().join(&options.input)
    }
  };
  let text = std::fs::read_to_string(input).map_err(|err| Error::from_reason(err.to_string()))?;
  let open_api =
    OpenAPIObject::from_str(&text).map_err(|err| Error::from_reason(err.to_string()))?;

  let command_factory =
    init_command_factory(&options.plugin).map_err(|err| Error::from_reason(err.to_string()))?;
  register_built_in_command(&command_factory.command);

  command_factory
    .command
    .execute_command(&options.command, &options.options, &open_api)
    .map_err(Error::from_reason)?;
  Ok(())
}

#[napi]
pub fn get_help_tree(options: Option<GetHelpTreeOptions>) -> Vec<HelpCommandDescriptor> {
  let plugin = options.and_then(|v| v.plugin);
  let command_factory = init_command_factory(&plugin).unwrap();
  register_built_in_command(&command_factory.command);
  command_factory
    .command
    .list_descriptors()
    .into_iter()
    .map(to_help_descriptor)
    .collect()
}
