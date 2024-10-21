use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use swagger_macro::schema_base_attributes;

use super::{SchemaEnum, SchemaTypeEnum};

#[schema_base_attributes]
#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaObject {
    #[serde(rename = "type")]
    pub r#type: SchemaTypeEnum,

    pub required: Option<Vec<String>>,

    pub properties: Option<HashMap<String, SchemaEnum>>,

    #[serde(rename = "additionalProperties")]
    pub additional_properties: Option<bool>,

    pub description: Option<String>,
}
