use antd::gen_files::gen_files;
use aptx_frontend_tk_binding_plugin::command::{
    CommandDescriptor, CommandRegistry, OptionDescriptor,
};

mod antd;

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
}
