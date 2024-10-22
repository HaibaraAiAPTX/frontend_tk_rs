use serde::{Deserialize, Serialize};
use swagger_macro::schema_base_attributes;

use super::SchemaTypeEnum;

#[schema_base_attributes]
#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaInteger {
    pub r#type: SchemaTypeEnum,

    pub format: Option<String>,

    pub description: Option<String>,

    pub minimum: Option<i32>,

    pub maximum: Option<i32>,

    #[serde(rename = "exclusiveMinimum")]
    pub exclusive_minimum: Option<bool>,

    #[serde(rename = "exclusiveMaximum")]
    pub exclusive_maximum: Option<bool>,

    #[serde(rename = "multipleOf")]
    pub multiple_of: Option<i32>,

    pub r#enum: Option<Vec<i32>>,
}
