use serde::{Deserialize, Serialize};

use super::ExternalDocumentationObject;

#[derive(Debug, Serialize, Deserialize)]
pub struct TagObject {
    pub name: String,

    pub description: Option<String>,

    #[serde(rename = "externalDocs")]
    pub external_docs: Option<ExternalDocumentationObject>,
}
