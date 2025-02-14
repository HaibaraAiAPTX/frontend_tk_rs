use libloading::{Error, Library, Symbol};
use swagger_gen::{
  built_in_api::{AxiosJsGen, AxiosTsGen, UniAppGen},
  core::GenRegistry,
};

pub fn init_gen_factory() -> GenRegistry {
  let factory = GenRegistry::default();

  factory.register("axios-ts", Box::new(|v| Box::new(AxiosTsGen::new(v))));
  factory.register("axios-js", Box::new(|v| Box::new(AxiosJsGen::new(v))));
  factory.register("uniapp", Box::new(|v| Box::new(UniAppGen::new(v))));

  factory
}

pub fn init_plugin(
  gen_factory: &GenRegistry,
  plugin: &Option<String>,
) -> Result<Option<Library>, Error> {
  if let Some(plugin) = plugin {
    unsafe {
      let lib = Library::new(plugin)?;
      let init_plugin_fn: Symbol<unsafe extern "C" fn(&GenRegistry)> = lib.get(b"init_plugin")?;
      init_plugin_fn(&gen_factory);
      Ok(Some(lib))
    }
  } else {
    Ok(None)
  }
}
