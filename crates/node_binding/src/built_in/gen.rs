use std::path::Path;

use aptx_frontend_tk_binding_plugin::utils::{create_all_file, ensure_path};
use clap::Parser;
use swagger_gen::{
  gen_api::{AxiosJsGen, AxiosTsGen, UniAppGen},
  gen_api_trait::GenApi,
  gen_declaration::TypescriptDeclarationGen,
};
use swagger_tk::model::OpenAPIObject;

/// 生成工具箱配置项
#[derive(Debug, Clone, Parser)]
pub struct ServiceModelsOps {
  /// 服务输出路径
  #[arg(long)]
  service_output: Option<Vec<String>>,

  /// service 模式
  #[arg(long)]
  service_mode: Option<Vec<String>>,

  /// 模型输出路径
  #[arg(long)]
  model_output: Option<Vec<String>>,
}

pub fn service_models(args: &[String], open_api: &OpenAPIObject) {
  let args: Vec<String> = std::iter::once("--".to_string())
    .chain(args.iter().cloned())
    .collect();
  let options = ServiceModelsOps::try_parse_from(args).unwrap();

  gen_apis(&options, open_api).unwrap();

  gen_models(&options, open_api).unwrap();
}

// 根据服务输出位置
fn gen_apis(options: &ServiceModelsOps, open_api: &OpenAPIObject) -> Result<(), String> {
  if let Some(outputs) = &options.service_output {
    let len = outputs.len();
    for i in 0..len {
      let mode = options
        .service_mode
        .as_ref()
        .and_then(|v| v.get(i).or_else(|| v.first()))
        .cloned()
        .unwrap_or("axios-ts".to_string());

      let mut gen_service = get_gen_service(&mode, open_api)?;
      gen_service.gen_apis()?;

      let apis = gen_service.get_outputs();
      let output = Path::new(outputs.get(i).unwrap());
      ensure_path(output);
      create_all_file(output, apis);
    }
  }
  Ok(())
}

fn get_gen_service<'a>(
  mode: &'a str,
  open_api: &'a OpenAPIObject,
) -> Result<Box<dyn GenApi + 'a>, String> {
  match mode {
    "axios-ts" => Ok(Box::new(AxiosTsGen::new(open_api))),
    "axios-js" => Ok(Box::new(AxiosJsGen::new(open_api))),
    "uniapp" => Ok(Box::new(UniAppGen::new(open_api))),
    _ => Err(format!("gen service mode not found, mode is {mode}")),
  }
}

fn gen_models(options: &ServiceModelsOps, open_api: &OpenAPIObject) -> Result<(), String> {
  if let Some(outputs) = &options.model_output {
    let model_gen = TypescriptDeclarationGen { open_api };
    let models = model_gen.gen_declarations()?;

    outputs.iter().for_each(|p| {
      let output = Path::new(p);
      ensure_path(output);
      create_all_file(output, &models);
    });
  }
  Ok(())
}
