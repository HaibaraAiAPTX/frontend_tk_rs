use crate::{
    getter::get_schema,
    model::{OpenAPIObject, ReferenceObject, SchemaEnum},
};

impl SchemaEnum {
    pub fn get_ts_type(&self) -> String {
        match self {
            SchemaEnum::Ref(schema) => schema.get_type_name(),
            SchemaEnum::Object(_schema) => "object".to_string(),
            SchemaEnum::String(_schema) => "string".to_string(),
            SchemaEnum::Integer(_schema) => "number".to_string(),
            SchemaEnum::Number(_schema) => "number".to_string(),
            SchemaEnum::Boolean(_schema) => "boolean".to_string(),
            SchemaEnum::Array(schema) => {
                let child_type = schema.items.as_ref().get_ts_type();
                format!("Array<{}>", child_type)
            }
        }
    }

    pub fn is_enum(&self, open_api: &OpenAPIObject) -> bool {
        match self {
            SchemaEnum::String(v) => v.r#enum.is_some(),
            SchemaEnum::Integer(v) => v.r#enum.is_some(),
            SchemaEnum::Number(v) => v.r#enum.is_some(),
            SchemaEnum::Ref(v) => {
                get_schema(open_api, &v.get_type_name()).map_or(false, |x| x.is_enum(open_api))
            }
            _ => false,
        }
    }

    pub fn can_be_null(&self, open_api: &OpenAPIObject) -> bool {
        match self {
            SchemaEnum::Ref(v) => {
                get_schema(open_api, &v.get_type_name()).map_or(false, |x| x.can_be_null(open_api))
            }
            SchemaEnum::Object(v) => v.nullable.unwrap_or_default(),
            SchemaEnum::String(v) => v.nullable.unwrap_or_default(),
            SchemaEnum::Integer(v) => v.nullable.unwrap_or_default(),
            SchemaEnum::Number(v) => v.nullable.unwrap_or_default(),
            SchemaEnum::Boolean(v) => v.nullable.unwrap_or_default(),
            SchemaEnum::Array(v) => v.nullable.unwrap_or_default(),
        }
    }
}

impl ReferenceObject {
    pub fn get_type_name(&self) -> String {
        self.r#ref.split("/").last().unwrap().to_string()
    }
}
