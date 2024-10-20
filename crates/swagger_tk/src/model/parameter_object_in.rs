use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ParameterObjectIn {
    #[serde(rename = "query")]
    Query,
    #[serde(rename = "header")]
    Header,
    #[serde(rename = "path")]
    Path,
    #[serde(rename = "cookie")]
    Cookie,
}
