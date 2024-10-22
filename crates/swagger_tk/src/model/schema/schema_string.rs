use serde::{Deserialize, Serialize};
use swagger_macro::schema_base_attributes;

use super::{SchemaStringFormat, SchemaTypeEnum};

#[schema_base_attributes]
#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaString {
    #[serde(rename = "type")]
    pub r#type: SchemaTypeEnum,

    pub format: Option<SchemaStringFormat>,

    pub description: Option<String>,

    pub pattern: Option<String>,

    #[serde(rename = "maxLength")]
    pub max_length: Option<i32>,

    #[serde(rename = "minLength")]
    pub min_length: Option<i32>,

    pub r#enum: Option<Vec<String>>,
}
