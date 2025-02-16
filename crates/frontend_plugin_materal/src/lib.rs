use antd::gen_files::gen_files;
use aptx_frontend_tk_binding_plugin::command::CommandRegistry;

mod antd;

#[no_mangle]
pub extern "C" fn init_plugin(command: &CommandRegistry) {
    command.register_command("materal-antd-init", Box::new(|args, open_api| {
        gen_files(args, open_api).unwrap();
    }));
}