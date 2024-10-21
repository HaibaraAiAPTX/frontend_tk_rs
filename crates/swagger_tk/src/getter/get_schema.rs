use crate::model::{OpenAPIObject, SchemaEnum};

pub fn get_schema<'a>(open_api_object: &'a OpenAPIObject, name: &str) -> Option<&'a SchemaEnum> {
    open_api_object.components.as_ref().and_then(|components| {
        components
            .schemas
            .as_ref()
            .and_then(|schemas| schemas.get(name))
    })
}
