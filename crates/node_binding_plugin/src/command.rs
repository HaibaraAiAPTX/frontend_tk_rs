use std::{cell::RefCell, collections::HashMap};

use swagger_tk::model::OpenAPIObject;

pub const COMMAND_DESCRIPTOR_SCHEMA_VERSION: &str = "1";

pub const COMMAND_DESCRIPTOR_FIELDS: &[&str] = &[
    "name",
    "summary",
    "description",
    "aliases",
    "options",
    "examples",
    "plugin_name",
    "plugin_version",
];

pub const OPTION_DESCRIPTOR_FIELDS: &[&str] = &[
    "long",
    "short",
    "value_name",
    "required",
    "multiple",
    "default_value",
    "description",
];

pub type CommandFn = Box<dyn for<'a> Fn(&'a [String], &'a OpenAPIObject)>;
pub type CommandError = String;

pub struct CommandContext<'a> {
    pub args: &'a [String],
    pub open_api: &'a OpenAPIObject,
}

pub trait CommandHandler: Send + Sync {
    fn descriptor(&self) -> CommandDescriptor;
    fn run(&self, ctx: CommandContext<'_>) -> Result<(), CommandError>;
}

#[derive(Debug, Clone, Default)]
pub struct OptionDescriptor {
    pub long: String,
    pub short: Option<char>,
    pub value_name: Option<String>,
    pub required: bool,
    pub multiple: bool,
    pub default_value: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Default)]
pub struct CommandDescriptor {
    pub name: String,
    pub summary: String,
    pub description: Option<String>,
    pub aliases: Vec<String>,
    pub options: Vec<OptionDescriptor>,
    pub examples: Vec<String>,
    pub plugin_name: Option<String>,
    pub plugin_version: Option<String>,
}

struct RegisteredCommand {
    descriptor: CommandDescriptor,
    callback: CommandFn,
}

#[derive(Default)]
pub struct CommandRegistry {
    command_map: RefCell<HashMap<String, RegisteredCommand>>,
}

impl CommandRegistry {
    /// 注册命令与完整元数据，供 help 系统使用
    pub fn register_command_with_descriptor(
        &self,
        descriptor: CommandDescriptor,
        callback: CommandFn,
    ) {
        let name = descriptor.name.clone();
        self.command_map.borrow_mut().insert(
            name,
            RegisteredCommand {
                descriptor,
                callback,
            },
        );
    }

    /// 获取所有命令元数据
    pub fn list_descriptors(&self) -> Vec<CommandDescriptor> {
        let mut result = self
            .command_map
            .borrow()
            .values()
            .map(|v| v.descriptor.clone())
            .collect::<Vec<_>>();
        result.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }

    pub fn execute_command<'a>(
        &self,
        name: &'a str,
        args: &'a [String],
        open_api: &'a OpenAPIObject,
    ) -> Result<(), String> {
        let map = self.command_map.borrow();
        let command = map.get(name).ok_or(format!("command not found: {name}"))?;
        (command.callback)(args, open_api);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_descriptor_contract_v1_should_match_snapshot() {
        assert_eq!(COMMAND_DESCRIPTOR_SCHEMA_VERSION, "1");
        assert_eq!(
            COMMAND_DESCRIPTOR_FIELDS,
            &[
                "name",
                "summary",
                "description",
                "aliases",
                "options",
                "examples",
                "plugin_name",
                "plugin_version",
            ]
        );
    }

    #[test]
    fn option_descriptor_contract_v1_should_match_snapshot() {
        assert_eq!(
            OPTION_DESCRIPTOR_FIELDS,
            &[
                "long",
                "short",
                "value_name",
                "required",
                "multiple",
                "default_value",
                "description",
            ]
        );
    }
}
