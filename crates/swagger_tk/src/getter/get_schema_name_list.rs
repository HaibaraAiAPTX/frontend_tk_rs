use crate::model::OpenAPIObject;

pub fn get_schema_name_list(open_api: &OpenAPIObject) -> Option<Vec<&String>> {
    Some(
        open_api
            .components
            .as_ref()?
            .schemas
            .as_ref()?
            .keys()
            .collect::<Vec<_>>(),
    )
}
