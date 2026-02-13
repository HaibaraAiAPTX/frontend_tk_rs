use antd::gen_files::gen_files;
use aptx_frontend_tk_binding_plugin::command::CommandDescriptor;
use enum_patch::export_materal_enum_patch;

mod antd;
mod enum_patch;

#[no_mangle]
pub extern "C" fn init_plugin(command: &aptx_frontend_tk_binding_plugin::command::CommandRegistry) {
    command.register_command_with_descriptor(
        CommandDescriptor {
            name: "materal:antd-init".to_string(),
            ..Default::default()
        },
        Box::new(|args, open_api| {
            gen_files(args, open_api).unwrap();
        }),
    );
    command.register_command_with_descriptor(
        CommandDescriptor {
            name: "materal:enum-patch".to_string(),
            ..Default::default()
        },
        Box::new(|args, open_api| {
            export_materal_enum_patch(args, open_api).unwrap();
        }),
    );
}
