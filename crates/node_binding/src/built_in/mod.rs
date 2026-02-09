use aptx_frontend_tk_binding_plugin::command::{CommandDescriptor, CommandRegistry};
pub mod gen;

/// 注册内置的命令
pub fn register_built_in_command(command: &CommandRegistry) {
  command.register_command_with_descriptor(
    CommandDescriptor {
      name: "gen".to_string(),
      summary: "Generate services and models from OpenAPI input".to_string(),
      description: Some("Built-in generator command (legacy entrypoint).".to_string()),
      examples: vec![
        "aptx-ft gen --service-output ./service --service-mode axios-ts --model-output ./models"
          .to_string(),
      ],
      plugin_name: Some("built-in".to_string()),
      plugin_version: Some(env!("CARGO_PKG_VERSION").to_string()),
      ..Default::default()
    },
    Box::new(gen::service_models),
  );
  frontend_plugin_materal::init_plugin(command);
}
