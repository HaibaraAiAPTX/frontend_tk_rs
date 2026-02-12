use aptx_frontend_tk_binding_plugin::command::{
  CommandDescriptor, CommandRegistry, OptionDescriptor,
};
pub mod ir;
pub mod model_enum_apply;
pub mod model_enum_plan;
pub mod model_gen;
pub mod model_ir;
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
      name: "model:gen".to_string(),
      summary: "Generate TypeScript model declarations from OpenAPI schemas".to_string(),
      description: Some(
        "Built-in model declaration generation command. Outputs *.d.ts/*.ts files.".to_string(),
      ),
      options: vec![
        OptionDescriptor {
          long: "output".to_string(),
          value_name: Some("dir".to_string()),
          required: true,
          description: "Output directory".to_string(),
          ..Default::default()
        },
        OptionDescriptor {
          long: "style".to_string(),
          value_name: Some("declaration|module".to_string()),
          default_value: Some("declaration".to_string()),
          description: "Output style: declaration(.d.ts) or module(export interface)".to_string(),
          ..Default::default()
        },
        OptionDescriptor {
          long: "name".to_string(),
          value_name: Some("schema".to_string()),
          multiple: true,
          description: "Generate specific schema names only; repeatable".to_string(),
          ..Default::default()
        },
      ],
      examples: vec![
        "aptx-ft model:gen --output ./generated/models".to_string(),
        "aptx-ft model:gen --output ./generated/models --style module".to_string(),
        "aptx-ft model:gen --output ./generated/models --name Order --name User".to_string(),
      ],
      plugin_name: Some("built-in".to_string()),
      plugin_version: Some(env!("CARGO_PKG_VERSION").to_string()),
      ..Default::default()
    },
    Box::new(model_gen::run_model_gen),
  );
  command.register_command_with_descriptor(
    CommandDescriptor {
      name: "model:ir".to_string(),
      summary: "Export model IR snapshot JSON from OpenAPI schemas".to_string(),
      description: Some("Built-in model intermediate representation export command.".to_string()),
      options: vec![OptionDescriptor {
        long: "output".to_string(),
        value_name: Some("file".to_string()),
        required: true,
        description: "Output JSON file path".to_string(),
        ..Default::default()
      }],
      examples: vec!["aptx-ft model:ir --output ./tmp/model-ir.json".to_string()],
      plugin_name: Some("built-in".to_string()),
      plugin_version: Some(env!("CARGO_PKG_VERSION").to_string()),
      ..Default::default()
    },
    Box::new(model_ir::export_model_ir_snapshot),
  );
  command.register_command_with_descriptor(
    CommandDescriptor {
      name: "model:enum-plan".to_string(),
      summary: "Export enum enrichment plan JSON from model IR".to_string(),
      description: Some(
        "Built-in enum enrichment planning command. Outputs enum candidates in a stable contract."
          .to_string(),
      ),
      options: vec![OptionDescriptor {
        long: "output".to_string(),
        value_name: Some("file".to_string()),
        required: true,
        description: "Output JSON file path".to_string(),
        ..Default::default()
      }],
      examples: vec!["aptx-ft model:enum-plan --output ./tmp/enum-plan.json".to_string()],
      plugin_name: Some("built-in".to_string()),
      plugin_version: Some(env!("CARGO_PKG_VERSION").to_string()),
      ..Default::default()
    },
    Box::new(model_enum_plan::export_model_enum_plan),
  );
  command.register_command_with_descriptor(
    CommandDescriptor {
      name: "model:enum-apply".to_string(),
      summary: "Apply enum patch file and generate model files".to_string(),
      description: Some(
        "Built-in enum enrichment apply command. Merges enum patch into model IR before rendering."
          .to_string(),
      ),
      options: vec![
        OptionDescriptor {
          long: "output".to_string(),
          value_name: Some("dir".to_string()),
          required: true,
          description: "Output directory".to_string(),
          ..Default::default()
        },
        OptionDescriptor {
          long: "patch".to_string(),
          value_name: Some("file".to_string()),
          required: true,
          description: "Enum patch JSON file path".to_string(),
          ..Default::default()
        },
        OptionDescriptor {
          long: "style".to_string(),
          value_name: Some("declaration|module".to_string()),
          default_value: Some("declaration".to_string()),
          description: "Output style: declaration(.d.ts) or module(export interface)".to_string(),
          ..Default::default()
        },
        OptionDescriptor {
          long: "conflict-policy".to_string(),
          value_name: Some("openapi-first|patch-first|provider-first".to_string()),
          default_value: Some("patch-first".to_string()),
          description: "Conflict policy when patch and OpenAPI enum members overlap".to_string(),
          ..Default::default()
        },
        OptionDescriptor {
          long: "name".to_string(),
          value_name: Some("schema".to_string()),
          multiple: true,
          description: "Generate specific schema names only; repeatable".to_string(),
          ..Default::default()
        },
      ],
      examples: vec![
        "aptx-ft model:enum-apply --patch ./tmp/enum-patch.json --output ./generated/models"
          .to_string(),
      ],
      plugin_name: Some("built-in".to_string()),
      plugin_version: Some(env!("CARGO_PKG_VERSION").to_string()),
      ..Default::default()
    },
    Box::new(model_enum_apply::run_model_enum_apply),
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
