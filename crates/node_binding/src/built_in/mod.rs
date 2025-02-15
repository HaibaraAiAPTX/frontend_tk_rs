use aptx_frontend_tk_binding_plugin::command::CommandRegistry;

pub mod gen;

/// 注册内置的命令
pub fn register_built_in_command(command: &CommandRegistry) {
  command.register_command("gen", Box::new(gen::service_models));
}
