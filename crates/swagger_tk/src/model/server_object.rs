use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::ServerVariableObject;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerObject {
    pub url: String,

    pub description: Option<String>,

    pub variables: Option<HashMap<String, ServerVariableObject>>,
}
