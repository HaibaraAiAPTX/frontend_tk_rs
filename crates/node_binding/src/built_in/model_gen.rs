use std::{fs, path::Path};

use aptx_frontend_tk_binding_plugin::utils::ensure_path;
use clap::Parser;
use swagger_gen::model_pipeline::{generate_model_files, ModelRenderStyle};
use swagger_tk::model::OpenAPIObject;

#[derive(Debug, Clone, Parser)]
pub struct ModelGenOps {
  #[arg(long)]
  output: String,

  #[arg(long, default_value = "module")]
  style: String,

  #[arg(long)]
  name: Option<Vec<String>>,
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
  let models = generate_model_files(open_api, style, &only_names).unwrap();
  for (name, content) in models {
    fs::write(output.join(name), content).unwrap();
  }
}
