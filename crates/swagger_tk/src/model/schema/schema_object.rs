use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use swagger_macro::schema_base_attributes;

use super::{SchemaEnum, SchemaObjectAdditionalProperties, SchemaTypeEnum};

#[schema_base_attributes]
#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaObject {
    #[serde(rename = "type")]
    pub r#type: SchemaTypeEnum,

    pub required: Option<Vec<String>>,

    pub properties: Option<HashMap<String, SchemaEnum>>,

    #[serde(rename = "additionalProperties")]
    pub additional_properties: Option<SchemaObjectAdditionalProperties>,

    pub description: Option<String>,

    #[serde(rename = "minProperties")]
    pub min_properties: Option<u32>,

    #[serde(rename = "maxProperties")]
    pub max_properties: Option<u32>,
}
