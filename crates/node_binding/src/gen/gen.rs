use std::path::Path;

use swagger_gen::{
  built_in_api_trait::GenApi, built_in_declaration::TypescriptDeclarationGen, core::GenRegistry,
};
use swagger_tk::model::OpenAPIObject;

use crate::{
  bootstrap::{init_gen_factory, init_plugin},
  utils::ensure_path,
};

/// 生成工具箱配置项
#[napi(object)]
#[derive(Debug, Clone)]
pub struct FrontendTkGenOps {
  /// swagger json 入口路径
  pub input: String,

  /// 插件路径
  pub plugin: Option<String>,

  /// 服务输出路径
  pub service_output: Option<Vec<String>>,

  /// 模型输出路径
  pub model_output: Option<Vec<String>>,

  /// service 模式
  pub service_mode: Option<Vec<String>>,
}

#[napi]
pub fn frontend_tk_gen(options: FrontendTkGenOps) {
  let text = std::fs::read_to_string(&options.input).unwrap();
  let open_api = OpenAPIObject::from_str(&text).unwrap();

  let gen_factory = init_gen_factory();
  #[allow(unused_variables)]
  let lib = init_plugin(&gen_factory, &options.plugin).unwrap();

  gen_apis(&options, &open_api, &gen_factory).unwrap();

  gen_models(&options, &open_api).unwrap();
}

/// 根据服务输出位置
fn gen_apis(
  options: &FrontendTkGenOps,
  open_api: &OpenAPIObject,
  gen_factory: &GenRegistry,
) -> Result<(), String> {
  if let Some(outputs) = &options.service_output {
    let len = outputs.len();
    for i in 0..len {
      // service mode
      let mode = options
        .service_mode
        .as_ref()
        .and_then(|v| v.get(i).or_else(|| v.first()))
        .cloned()
        .unwrap_or("axios-ts".to_string());

      let gen_service = gen_factory
        .create(&mode, &open_api)
        .ok_or("get gen service fail")?;

      gen_api(gen_service, Path::new(outputs.get(i).unwrap()))?;
    }
  }
  Ok(())
}

fn gen_api<'a>(mut service_gen: Box<dyn GenApi + '_>, output: &Path) -> Result<(), String> {
  service_gen.gen_apis().unwrap();
  let apis = service_gen.get_outputs();

  ensure_path(output);

  for (name, content) in apis {
    let file_path = output.join(name);
    std::fs::write(file_path, content.clone()).unwrap();
  }

  Ok(())
}

fn gen_models(options: &FrontendTkGenOps, open_api: &OpenAPIObject) -> Result<(), String> {
  if let Some(outputs) = &options.model_output {
    let model_gen = TypescriptDeclarationGen { open_api };
    let models = model_gen.gen_declarations()?;

    outputs.iter().for_each(|p| {
      let output = Path::new(p);
      ensure_path(&output);

      for (name, content) in &models {
        let file_path = output.join(name);
        std::fs::write(file_path, content).unwrap();
      }
    });
  }
  Ok(())
}
