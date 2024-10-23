use serde::{self, Deserialize, Serialize};
use swagger_macro::schema_base_attributes;

use super::{SchemaEnum, SchemaTypeEnum};

#[schema_base_attributes]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchemaArray {
    #[serde(rename = "type")]
    pub r#type: SchemaTypeEnum,

    pub items: Box<SchemaEnum>,

    /// 是否唯一
    #[serde(rename = "uniqueItems")]
    pub unique_items: Option<bool>,
}
