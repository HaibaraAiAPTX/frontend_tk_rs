use aptx_frontend_tk_binding_plugin::command::{
  CommandDescriptor, CommandRegistry, OptionDescriptor,
};
pub mod ir;
pub mod terminal_codegen;

/// 注册内置的命令
pub fn register_built_in_command(command: &CommandRegistry) {
  command.register_command_with_descriptor(
    CommandDescriptor {
      name: "terminal:codegen".to_string(),
      summary: "Generate output for one built-in terminal from OpenAPI input".to_string(),
      description: Some("Built-in IR terminal generation command.".to_string()),
      options: vec![
        OptionDescriptor {
          long: "terminal".to_string(),
          value_name: Some("id".to_string()),
          required: true,
          description: "Terminal id, e.g. axios-ts/react-query".to_string(),
          ..Default::default()
        },
        OptionDescriptor {
          long: "output".to_string(),
          value_name: Some("dir".to_string()),
          required: true,
          description: "Output directory".to_string(),
          ..Default::default()
        },
      ],
      examples: vec![
        "aptx-ft terminal:codegen --terminal axios-ts --output ./generated/services/axios-ts"
          .to_string(),
      ],
      plugin_name: Some("built-in".to_string()),
      plugin_version: Some(env!("CARGO_PKG_VERSION").to_string()),
      ..Default::default()
    },
    Box::new(terminal_codegen::run_terminal_codegen),
  );
  command.register_command_with_descriptor(
    CommandDescriptor {
      name: "ir:snapshot".to_string(),
      summary: "Export IR snapshot JSON from OpenAPI input".to_string(),
      description: Some(
        "Built-in IR export command used by script renderers/commands.".to_string(),
      ),
      options: vec![OptionDescriptor {
        long: "output".to_string(),
        value_name: Some("file".to_string()),
        required: true,
        description: "Output JSON file path".to_string(),
        ..Default::default()
      }],
      examples: vec!["aptx-ft ir:snapshot --output ./tmp/ir.json".to_string()],
      plugin_name: Some("built-in".to_string()),
      plugin_version: Some(env!("CARGO_PKG_VERSION").to_string()),
      ..Default::default()
    },
    Box::new(ir::export_ir_snapshot),
  );
  frontend_plugin_materal::init_plugin(command);
}
