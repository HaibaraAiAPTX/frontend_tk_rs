use crate::model::OpenAPIObject;

pub fn get_schema_name_list(open_api_object: &OpenAPIObject) -> Option<Vec<&String>> {
    open_api_object.components.as_ref().and_then(|components| {
        components
            .schemas
            .as_ref()
            .map(|schemas| schemas.keys().collect())
    })
}
