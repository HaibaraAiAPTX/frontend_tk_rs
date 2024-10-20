use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct HeaderObject {
    pub name: Option<String>,

    pub description: Option<String>,

    pub required: Option<bool>,

    pub deprecated: Option<bool>,

    #[serde(rename = "allowEmptyValue")]
    pub allow_empty_value: Option<bool>,
}
