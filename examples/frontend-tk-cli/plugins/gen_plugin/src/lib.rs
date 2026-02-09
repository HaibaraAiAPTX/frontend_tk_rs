use aptx_frontend_tk_binding_plugin::command::{CommandDescriptor, CommandRegistry};

#[no_mangle]
pub extern "C" fn init_plugin(command: &CommandRegistry) {
    command.register_command_with_descriptor(
        CommandDescriptor {
            name: "custom-command".to_string(),
            summary: "Run demo custom plugin command".to_string(),
            plugin_name: Some("gen_plugin".to_string()),
            plugin_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            ..Default::default()
        },
        Box::new(|_a, _b| println!("成功执行了自定义命令")),
    );
}
