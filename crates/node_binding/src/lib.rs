#![deny(clippy::all)]

use std::{env::current_dir, path::Path, str::FromStr};

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
  pub input: Option<String>,
  pub command: String,
  pub plugin: Option<Vec<String>>,
  pub options: Vec<String>,
}

#[napi]
pub fn run_cli(options: RunCliOptions) -> napi::Result<()> {
  // Parse OpenAPI only when input is provided
  let open_api = if let Some(input_path) = &options.input {
    let path = Path::new(input_path);
    let abs_path = if path.is_absolute() {
      path.to_path_buf()
    } else {
      current_dir().unwrap().join(input_path)
    };
    let text = std::fs::read_to_string(&abs_path)
      .map_err(|err| Error::from_reason(err.to_string()))?;
    OpenAPIObject::from_str(&text)
      .map_err(|err| Error::from_reason(err.to_string()))?
  } else {
    // Create a minimal valid OpenAPIObject for commands that don't need it
    OpenAPIObject {
      openapi: "3.0.0".to_string(),
      info: None,
      servers: None,
      paths: None,
      webhooks: None,
      components: None,
      security: None,
      tags: None,
      external_docs: None,
    }
  };

  let command_factory =
    init_command_factory(&options.plugin).map_err(|err| Error::from_reason(err.to_string()))?;
  register_built_in_command(&command_factory.command);

  command_factory
    .command
    .execute_command(&options.command, &options.options, &open_api)
    .map_err(Error::from_reason)?;
  Ok(())
}
