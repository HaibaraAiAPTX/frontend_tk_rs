#![deny(clippy::all)]

use std::{env::current_dir, path::Path, str::FromStr};

use bootstrap::init_command_factory;
use built_in::register_built_in_command;
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

#[napi]
pub fn run_cli(options: RunCliOptions) {
  let input = {
    let path = Path::new(&options.input);
    if path.is_absolute() {
      path.to_path_buf()
    } else {
      current_dir().unwrap().join(&options.input)
    }
  };
  let text = std::fs::read_to_string(input).unwrap();
  let open_api = OpenAPIObject::from_str(&text).unwrap();

  let command_factory = init_command_factory(&options.plugin).unwrap();
  register_built_in_command(&command_factory.command);

  command_factory
    .command
    .excute_command(&options.command, &options.options, &open_api)
    .unwrap();
}
