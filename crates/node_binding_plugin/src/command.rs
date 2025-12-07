use std::{cell::RefCell, collections::HashMap};

use swagger_tk::model::OpenAPIObject;

#[derive(Default)]
pub struct CommandRegistry {
    command_map: RefCell<HashMap<String, Box<dyn for<'a> Fn(&'a Vec<String>, &'a OpenAPIObject)>>>,
}

impl CommandRegistry {
    pub fn register_command(
        &self,
        name: &str,
        callback: Box<dyn for<'a> Fn(&'a Vec<String>, &'a OpenAPIObject)>,
    ) {
        self.command_map
            .borrow_mut()
            .insert(name.to_string(), callback);
    }

    pub fn excute_command<'a>(
        &self,
        name: &'a str,
        args: &'a Vec<String>,
        open_api: &'a OpenAPIObject,
    ) -> Result<(), String> {
        let map = self.command_map.borrow();
        let callback = map.get(name).ok_or(format!("command not found: {name}"))?;
        callback(args, open_api);
        Ok(())
    }
}
