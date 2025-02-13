#![deny(clippy::all)]
use get_service::{get_gen_service_by_string, load_gen_service_plugin};
use std::{fs, path::Path};
use swagger_gen::{built_in_api_trait::GenApi, built_in_declaration::TypescriptDeclarationGen, utils::format_ts_code};
use swagger_tk::model::OpenAPIObject;

mod get_service;

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

  /// 自定义服务生成器路径
  pub gen_service_plugin: Option<String>,
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

    if let Some(path) = options.gen_service_plugin {
      let service = load_gen_service_plugin(&path);
      gen_api(service.service, &open_api, service_output)
    } else {
      let mode = options.service_mode.as_ref().map(|v| v.as_str()).unwrap_or("axios");
      let service = get_gen_service_by_string(mode).unwrap();
      gen_api(service, &open_api, service_output)
    }
  }

  if let Some(output) = &options.model_output {
    gen_model(&open_api, Path::new(output));
  }
}

fn gen_api<'a>(
  mut service_gen: Box<dyn GenApi<'a> + '_>,
  open_api: &'a OpenAPIObject,
  outputs: Vec<&Path>,
) {
  service_gen.set_open_api(open_api);
  service_gen.gen_apis(&open_api).unwrap();

  let apis = service_gen.get_outputs();

  outputs.iter().for_each(|&o| {
    ensure_path(o);
  });

  for (name, content) in apis {
    let content = format_ts_code(&content).unwrap();
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
