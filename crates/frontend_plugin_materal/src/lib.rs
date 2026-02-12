use antd::gen_files::gen_files;
use aptx_frontend_tk_binding_plugin::command::{
    CommandDescriptor, CommandRegistry, OptionDescriptor,
};
use enum_patch::export_materal_enum_patch;

mod antd;
mod enum_patch;

#[no_mangle]
pub extern "C" fn init_plugin(command: &CommandRegistry) {
    command.register_command_with_descriptor(
        CommandDescriptor {
            name: "materal:antd-init".to_string(),
            summary: "Generate Ant Design material scaffold from OpenAPI".to_string(),
            description: Some("Frontend material plugin command.".to_string()),
            options: vec![OptionDescriptor {
                long: "store".to_string(),
                short: None,
                value_name: Some("boolean".to_string()),
                required: false,
                multiple: false,
                default_value: None,
                description: "Whether to generate dictionary store".to_string(),
            }],
            examples: vec!["aptx-ft materal:antd-init -i ./openapi.json --store true".to_string()],
            plugin_name: Some("frontend_plugin_materal".to_string()),
            plugin_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            ..Default::default()
        },
        Box::new(|args, open_api| {
            gen_files(args, open_api).unwrap();
        }),
    );
    command.register_command_with_descriptor(
        CommandDescriptor {
            name: "materal:enum-patch".to_string(),
            summary: "Fetch Materal enum values and output EnumPatch JSON".to_string(),
            description: Some(
                "Detects /Enums/GetAll* paths, fetches enum values from Materal API and writes standard enum patch contract."
                    .to_string(),
            ),
            options: vec![
                OptionDescriptor {
                    long: "base-url".to_string(),
                    short: None,
                    value_name: Some("url".to_string()),
                    required: true,
                    multiple: false,
                    default_value: None,
                    description: "Materal API base URL".to_string(),
                },
                OptionDescriptor {
                    long: "output".to_string(),
                    short: None,
                    value_name: Some("file".to_string()),
                    required: true,
                    multiple: false,
                    default_value: None,
                    description: "Output patch JSON file path".to_string(),
                },
                OptionDescriptor {
                    long: "max-retries".to_string(),
                    short: None,
                    value_name: Some("n".to_string()),
                    required: false,
                    multiple: false,
                    default_value: Some("3".to_string()),
                    description: "HTTP retry times".to_string(),
                },
                OptionDescriptor {
                    long: "timeout-ms".to_string(),
                    short: None,
                    value_name: Some("ms".to_string()),
                    required: false,
                    multiple: false,
                    default_value: Some("10000".to_string()),
                    description: "HTTP timeout milliseconds".to_string(),
                },
                OptionDescriptor {
                    long: "naming-strategy".to_string(),
                    short: None,
                    value_name: Some("auto|none".to_string()),
                    required: false,
                    multiple: false,
                    default_value: Some("auto".to_string()),
                    description: "Suggested enum member naming strategy".to_string(),
                },
            ],
            examples: vec![
                "aptx-ft -i ./openapi.json materal:enum-patch --base-url http://localhost:5000 --output ./tmp/enum-patch.json"
                    .to_string(),
            ],
            plugin_name: Some("frontend_plugin_materal".to_string()),
            plugin_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            ..Default::default()
        },
        Box::new(|args, open_api| {
            export_materal_enum_patch(args, open_api).unwrap();
        }),
    );
}
