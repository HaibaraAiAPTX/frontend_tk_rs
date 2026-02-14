use aptx_frontend_tk_binding_plugin::command::{CommandDescriptor, CommandRegistry};
pub mod aptx_commands;
pub mod ir;
pub mod model_enum_apply;
pub mod model_enum_plan;
pub mod model_gen;
pub mod model_ir;
pub mod std_commands;
pub mod terminal_codegen;

/// 注册内置的命令
pub fn register_built_in_command(command: &CommandRegistry) {
  command.register_command_with_descriptor(
    CommandDescriptor {
      name: "terminal:codegen".to_string(),
      ..Default::default()
    },
    Box::new(terminal_codegen::run_terminal_codegen),
  );
  command.register_command_with_descriptor(
    CommandDescriptor {
      name: "model:gen".to_string(),
      ..Default::default()
    },
    Box::new(model_gen::run_model_gen),
  );
  command.register_command_with_descriptor(
    CommandDescriptor {
      name: "model:ir".to_string(),
      ..Default::default()
    },
    Box::new(model_ir::export_model_ir_snapshot),
  );
  command.register_command_with_descriptor(
    CommandDescriptor {
      name: "model:enum-plan".to_string(),
      ..Default::default()
    },
    Box::new(model_enum_plan::export_model_enum_plan),
  );
  command.register_command_with_descriptor(
    CommandDescriptor {
      name: "model:enum-apply".to_string(),
      ..Default::default()
    },
    Box::new(model_enum_apply::run_model_enum_apply),
  );
  command.register_command_with_descriptor(
    CommandDescriptor {
      name: "ir:snapshot".to_string(),
      ..Default::default()
    },
    Box::new(ir::export_ir_snapshot),
  );

  // Register @aptx namespace commands
  command.register_command_with_descriptor(
    CommandDescriptor {
      name: "aptx:functions".to_string(),
      ..Default::default()
    },
    Box::new(aptx_commands::run_aptx_functions),
  );

  command.register_command_with_descriptor(
    CommandDescriptor {
      name: "aptx:react-query".to_string(),
      ..Default::default()
    },
    Box::new(aptx_commands::run_aptx_react_query),
  );

  command.register_command_with_descriptor(
    CommandDescriptor {
      name: "aptx:vue-query".to_string(),
      ..Default::default()
    },
    Box::new(aptx_commands::run_aptx_vue_query),
  );

  // Register std namespace commands
  command.register_command_with_descriptor(
    CommandDescriptor {
      name: "std:axios-ts".to_string(),
      ..Default::default()
    },
    Box::new(std_commands::run_std_axios_ts),
  );

  command.register_command_with_descriptor(
    CommandDescriptor {
      name: "std:axios-js".to_string(),
      ..Default::default()
    },
    Box::new(std_commands::run_std_axios_js),
  );

  command.register_command_with_descriptor(
    CommandDescriptor {
      name: "std:uniapp".to_string(),
      ..Default::default()
    },
    Box::new(std_commands::run_std_uniapp),
  );

  frontend_plugin_materal::init_plugin(command);
}
