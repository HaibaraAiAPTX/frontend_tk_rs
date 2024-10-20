use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SecuritySchemeIn {
    #[serde(rename = "query")]
    Query,
    #[serde(rename = "header")]
    Header,
    #[serde(rename = "cookie")]
    Cookie,
}
