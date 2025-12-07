use crate::model::{OpenAPIObject, SchemaEnum};

use super::get_all_schema;

/// 根据 name 获取单个的 schema
pub fn get_schema_by_name<'a>(open_api: &'a OpenAPIObject, name: &str) -> Option<&'a SchemaEnum> {
    get_all_schema(open_api)?.get(name)
}
