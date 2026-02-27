use std::{fs, path::Path};

use aptx_frontend_tk_binding_plugin::utils::ensure_path;
use clap::Parser;
use swagger_gen::model_pipeline::{generate_model_files, generate_model_files_with_existing, ModelRenderStyle};
use swagger_tk::model::OpenAPIObject;

use super::model_enum_plan::load_existing_enums_from_model_files;

#[derive(Debug, Clone, Parser)]
pub struct ModelGenOps {
  #[arg(long)]
  output: String,

  #[arg(long, default_value = "module")]
  style: String,

  #[arg(long)]
  name: Option<Vec<String>>,

  #[arg(long, default_value = "false")]
  preserve: bool,
}

pub fn run_model_gen(args: &[String], open_api: &OpenAPIObject) {
  let args: Vec<String> = std::iter::once("--".to_string())
    .chain(args.iter().cloned())
    .collect();
  let options = ModelGenOps::try_parse_from(args).unwrap();
  let output = Path::new(&options.output);
  ensure_path(output);
  let style = ModelRenderStyle::parse(&options.style).unwrap();
  let only_names = options.name.unwrap_or_default();

  let models = if options.preserve {
    // Load existing enums from output directory
    let existing_enums = load_existing_enums_from_model_files(output);
    match existing_enums {
      Some(enums) => {
        generate_model_files_with_existing(open_api, style, &only_names, &enums).unwrap()
      }
      None => {
        generate_model_files(open_api, style, &only_names).unwrap()
      }
    }
  } else {
    generate_model_files(open_api, style, &only_names).unwrap()
  };

  for (name, content) in models {
    fs::write(output.join(name), content).unwrap();
  }
}
