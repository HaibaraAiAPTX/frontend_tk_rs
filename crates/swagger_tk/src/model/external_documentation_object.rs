use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalDocumentationObject {
    pub description: Option<String>,

    pub url: String,
}
