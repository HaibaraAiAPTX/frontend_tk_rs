use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use swagger_macro::{self, schema_base_attributes};

use super::{SchemaEnum, SchemaTypeEnum};

#[derive(Debug, Serialize, Deserialize)]
#[schema_base_attributes]
pub struct SchemaObject {
    #[serde(rename = "type")]
    pub r#type: SchemaTypeEnum,

    pub properties: Option<HashMap<String, SchemaEnum>>,

    #[serde(rename = "additionalProperties")]
    pub additional_properties: Option<bool>,

    pub description: Option<String>,
}
