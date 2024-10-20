use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthFlowObject {
    #[serde(rename = "authorizationUrl")]
    pub authorization_url: String,

    #[serde(rename = "tokenUrl")]
    pub token_url: String,

    #[serde(rename = "refreshUrl")]
    pub refresh_url: Option<String>,

    pub scopes: HashMap<String, String>,
}
