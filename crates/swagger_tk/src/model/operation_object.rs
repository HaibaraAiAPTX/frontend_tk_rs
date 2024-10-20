use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{CallbackValue, ExternalDocumentationObject, OperationObjectParameters, OperationObjectRequestBody, ResponsesValue, ServerObject};

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationObject {
    pub tags: Option<Vec<String>>,

    pub summary: Option<String>,

    pub description: Option<String>,

    #[serde(rename = "externalDocs")]
    pub external_docs: Option<ExternalDocumentationObject>,

    #[serde(rename = "operationId")]
    pub operation_id: Option<String>,

    pub parameters: Option<Vec<OperationObjectParameters>>,

    #[serde(rename = "requestBody")]
    pub request_body: Option<OperationObjectRequestBody>,

    pub responses: Option<HashMap<String, ResponsesValue>>,

    pub callbacks: Option<HashMap<String, CallbackValue>>,

    pub deprecated: Option<bool>,

    pub security: Option<Vec<HashMap<String, Vec<String>>>>,

    pub servers: Option<Vec<ServerObject>>,
}
