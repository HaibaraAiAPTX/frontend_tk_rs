use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ReferenceObject {
    #[serde(rename = "$ref")]
    pub r#ref: String,

    pub summary: Option<String>,

    pub description: Option<String>,
}
