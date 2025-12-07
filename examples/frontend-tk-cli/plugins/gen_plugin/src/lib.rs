use aptx_frontend_tk_binding_plugin::command::CommandRegistry;

#[no_mangle]
pub extern "C" fn init_plugin(command: &CommandRegistry) {
    command.register_command(
        "custom-command",
        Box::new(|_a, _b| println!("成功执行了自定义命令")),
    );
}
