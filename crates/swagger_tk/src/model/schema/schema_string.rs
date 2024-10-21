use serde::{Deserialize, Serialize};
use swagger_macro::{self, schema_base_attributes};

use super::SchemaTypeEnum;

#[derive(Debug, Serialize, Deserialize)]
#[schema_base_attributes]
pub struct SchemaString {
    #[serde(rename = "type")]
    pub r#type: SchemaTypeEnum,
}