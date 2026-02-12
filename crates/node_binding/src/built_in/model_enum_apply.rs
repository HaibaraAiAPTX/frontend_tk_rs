use std::{fs, path::Path};

use aptx_frontend_tk_binding_plugin::utils::ensure_path;
use clap::Parser;
use swagger_gen::model_pipeline::{
  generate_model_files_with_enum_patch, EnumConflictPolicy, EnumPatch, EnumPatchDocument,
  ModelRenderStyle,
};
use swagger_tk::model::OpenAPIObject;

#[derive(Debug, Clone, Parser)]
pub struct ModelEnumApplyOps {
  #[arg(long)]
  output: String,

  #[arg(long)]
  patch: String,

  #[arg(long, default_value = "declaration")]
  style: String,

  #[arg(long, default_value = "patch-first")]
  conflict_policy: String,

  #[arg(long)]
  name: Option<Vec<String>>,
}

pub fn run_model_enum_apply(args: &[String], open_api: &OpenAPIObject) {
  let args: Vec<String> = std::iter::once("--".to_string())
    .chain(args.iter().cloned())
    .collect();
  let options = ModelEnumApplyOps::try_parse_from(args).unwrap();
  let output = Path::new(&options.output);
  ensure_path(output);
  let style = ModelRenderStyle::parse(&options.style).unwrap();
  let conflict_policy = EnumConflictPolicy::parse(&options.conflict_policy).unwrap();
  let only_names = options.name.unwrap_or_default();
  let patches = read_patches(&options.patch).unwrap();
  let models =
    generate_model_files_with_enum_patch(open_api, style, &only_names, &patches, conflict_policy)
      .unwrap();
  for (name, content) in models {
    fs::write(output.join(name), content).unwrap();
  }
}

fn read_patches(path: &str) -> Result<Vec<EnumPatch>, String> {
  let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
  if let Ok(doc) = serde_json::from_str::<EnumPatchDocument>(&text) {
    return Ok(doc.patches);
  }
  serde_json::from_str::<Vec<EnumPatch>>(&text).map_err(|err| err.to_string())
}
