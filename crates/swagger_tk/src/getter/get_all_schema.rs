use std::collections::HashMap;

use crate::model::{OpenAPIObject, SchemaEnum};

/// 获取完整的 schema 对象
pub fn get_all_schema(open_api: &OpenAPIObject) -> Option<&HashMap<String, SchemaEnum>> {
    open_api.components.as_ref()?.schemas.as_ref()
}
