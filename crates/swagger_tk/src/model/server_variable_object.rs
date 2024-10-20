use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerVariableObject {
    #[serde(rename = "enum")]
    pub r#enum: Vec<String>,

    pub default: String,

    pub description: Option<String>,
}
