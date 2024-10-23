use serde::{Deserialize, Serialize};
use swagger_macro::schema_base_attributes;

use super::SchemaTypeEnum;

#[schema_base_attributes]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchemaBool {
    #[serde(rename = "type")]
    pub r#type: SchemaTypeEnum,
}
