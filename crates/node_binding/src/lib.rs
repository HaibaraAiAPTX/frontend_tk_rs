#![deny(clippy::all)]
use std::{fs, path::Path};

use swagger_gen::{
  build_in_api_trait::GenApi,
  built_in_api::{AxiosTsGen, UniAppGen},
  built_in_declaration::TypescriptDeclarationGen,
};
use swagger_tk::model::OpenAPIObject;

#[macro_use]
extern crate napi_derive;

/// 生成工具箱配置项
#[napi(object)]
#[derive(Debug)]
pub struct FrontendTkGenOps {
  /// swagger json 入口路径
  pub input: String,

  /// 服务输出路径
  pub service_output: Option<Vec<String>>,

  /// 模型输出路径
  pub model_output: Option<String>,

  /// service 模式
  pub service_mode: Option<String>,

  /// 是否生成基础文件
  pub gen_base_service: Option<bool>,
}

#[napi]
pub fn frontend_tk_gen(options: FrontendTkGenOps) {
  let text = std::fs::read_to_string(&options.input).unwrap();
  let open_api = OpenAPIObject::from_str(&text).unwrap();

  if let Some(outpus) = &options.service_output {
    let service_output = outpus
      .iter()
      .map(|path_str| Path::new(path_str))
      .collect::<Vec<&Path>>();
    gen_api(
      &open_api,
      options
        .service_mode
        .as_ref()
        .unwrap_or(&"axios".to_string()),
      options.gen_base_service,
      service_output,
    );
  }

  if let Some(output) = &options.model_output {
    gen_model(&open_api, Path::new(output));
  }
}

fn get_gen_by_string(mode: &str) -> Result<Box<dyn GenApi + '_>, String> {
  match mode {
    "axios" => Ok(Box::new(AxiosTsGen::default())),
    "uniapp" => Ok(Box::new(UniAppGen::default())),
    _ => Err(format!("未知的生成模式: {}", mode)),
  }
}

fn gen_api(open_api: &OpenAPIObject, mode: &str, gen_base: Option<bool>, outputs: Vec<&Path>) {
  let mut service_gen = get_gen_by_string(mode).unwrap();
  service_gen.set_open_api(open_api);
  service_gen.gen_apis(&open_api).unwrap();
  if let Some(true) = gen_base {
    service_gen.gen_base_service();
  }

  let apis = service_gen.get_outputs();

  outputs.iter().for_each(|&o| {
    ensure_path(o);
  });

  for (name, content) in apis {
    for output in &outputs {
      let file_path = output.join(name);
      std::fs::write(file_path, content.clone()).unwrap();
    }
  }
}

fn gen_model(open_api: &OpenAPIObject, output: &Path) {
  let model_gen = TypescriptDeclarationGen { open_api };

  ensure_path(&output);

  let models = model_gen.gen_declarations();
  if let Ok(models) = models {
    for (name, content) in models {
      let file_path = output.join(name);
      std::fs::write(file_path, content).unwrap();
    }
  }
}

fn ensure_path(p: &Path) {
  if !p.exists() {
    fs::create_dir(p).unwrap()
  }
}
