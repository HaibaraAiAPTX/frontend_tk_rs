use std::{fs, path::Path};

use aptx_frontend_tk_binding_plugin::utils::ensure_path;
use clap::Parser;
use swagger_gen::gen_declaration::TypescriptDeclarationGen;
use swagger_tk::model::OpenAPIObject;

#[derive(Debug, Clone, Parser)]
pub struct ModelGenOps {
  #[arg(long)]
  output: String,

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

  let model_gen = TypescriptDeclarationGen { open_api };

  if let Some(names) = options.name {
    for model_name in names {
      let (content, is_enum) = model_gen.gen_declaration_by_name(&model_name).unwrap();
      let file_name = if is_enum {
        format!("{model_name}.ts")
      } else {
        format!("{model_name}.d.ts")
      };
      fs::write(output.join(file_name), content).unwrap();
    }
    return;
  }

  let models = model_gen.gen_declarations().unwrap();
  for (name, content) in models {
    fs::write(output.join(name), content).unwrap();
  }
}
