use serde::{Deserialize, Serialize};

use super::{ParameterObjectIn, SchemaEnum};

#[derive(Debug, Deserialize, Serialize)]
pub struct ParameterObject {
    pub name: String,

    pub r#in: ParameterObjectIn,

    pub description: Option<String>,

    pub required: Option<bool>,

    pub deprecated: Option<bool>,

    #[serde(rename = "allowEmptyValue")]
    pub allow_empty_value: Option<bool>,

    pub style: Option<String>,

    pub explode: Option<bool>,

    #[serde(rename = "allowReserved")]
    pub allow_reserved: Option<bool>,

    pub schema: Option<SchemaEnum>,
}
