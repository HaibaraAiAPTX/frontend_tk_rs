use aptx_frontend_tk_binding_plugin::command::CommandRegistry;
use libloading::{Error, Library, Symbol};

#[derive(Default)]
pub(crate) struct CommandFactory {
  libs: Vec<Library>,
  pub command: CommandRegistry,
}

impl CommandFactory {
  fn insert_lib(&mut self, lib: Library) {
    self.libs.push(lib);
  }
}

pub fn init_command_factory(plugin: &Option<Vec<String>>) -> Result<CommandFactory, Error> {
  let mut command_factory = CommandFactory::default();
  plugin.as_ref().map(|v| {
    v.iter().for_each(|p| unsafe {
      let lib = Library::new(p).unwrap();
      let init_plugin: Symbol<unsafe extern "C" fn(&CommandRegistry)> = lib.get(b"init_plugin").unwrap();
      init_plugin(&command_factory.command);
      command_factory.insert_lib(lib);
    })
  });
  Ok(command_factory)
}
