use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{ComponentsObject, OpenAPIWebhooks};

use super::{ExternalDocumentationObject, OpenAPIInfo, PathItemObject, ServerObject, TagObject};

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAPIObject {
    /// 版本号
    pub openapi: String,

    /// 信息
    pub info: Option<OpenAPIInfo>,

    // pub json_schema_dialect: String,
    pub servers: Option<Vec<ServerObject>>,

    pub paths: Option<HashMap<String, PathItemObject>>,

    pub webhooks: Option<HashMap<String, OpenAPIWebhooks>>,

    pub components: Option<ComponentsObject>,
    
    pub security: Option<Vec<HashMap<String, Vec<String>>>>,

    pub tags: Option<Vec<TagObject>>,

    #[serde(rename = "externalDocs")]
    pub external_docs: Option<ExternalDocumentationObject>,
}

impl OpenAPIObject {
    pub fn from_str(swagger_text: &str) -> Result<Self, serde_json::Error> {
        let open_api_object: OpenAPIObject = serde_json::from_str(swagger_text)?;
        Ok(open_api_object)
    }
}
