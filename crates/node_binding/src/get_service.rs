use libloading::{Library, Symbol};
use swagger_gen::{
  built_in_api::{AxiosTsGen, UniAppGen},
  built_in_api_trait::GenApi,
};

// 添加一个结构体来持有库和服务
pub struct PluginService<'a> {
  _lib: Library, // 保持库活着
  pub service: Box<dyn GenApi<'a>>,
}

pub fn load_gen_service_plugin<'a>(plugin_path: &str) -> PluginService<'a> {
  unsafe {
    // 加载动态库
    let lib = Library::new(plugin_path).unwrap(); // 根据系统调整文件名
    let create_plugin: Symbol<unsafe extern "C" fn() -> Box<dyn GenApi<'a>>> =
      lib.get(b"create_plugin").unwrap();
    let service = create_plugin();
    PluginService { _lib: lib, service }
  }
}

pub fn get_gen_service_by_string(mode: &str) -> Result<Box<dyn GenApi + '_>, String> {
  match mode {
    "axios" => Ok(Box::new(AxiosTsGen::default())),
    "uniapp" => Ok(Box::new(UniAppGen::default())),
    _ => Err(format!("未知的生成模式: {}", mode)),
  }
}