use std::{fs, path::Path};

use aptx_frontend_tk_binding_plugin::utils::ensure_path;
use clap::Parser;
use swagger_gen::model_pipeline::build_model_enum_plan_json;
use swagger_tk::model::OpenAPIObject;

#[derive(Debug, Clone, Parser)]
pub struct ModelEnumPlanOps {
  #[arg(long)]
  output: String,
}

pub fn export_model_enum_plan(args: &[String], open_api: &OpenAPIObject) {
  let args: Vec<String> = std::iter::once("--".to_string())
    .chain(args.iter().cloned())
    .collect();
  let options = ModelEnumPlanOps::try_parse_from(args).unwrap();

  let output = Path::new(&options.output);
  if let Some(parent) = output.parent() {
    ensure_path(parent);
  }

  let json = build_model_enum_plan_json(open_api).unwrap();
  fs::write(output, json).unwrap();
}
