use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAPIInfoLicense {
    pub name: String,

    pub identifier: Option<String>,

    pub url: Option<String>,
}
