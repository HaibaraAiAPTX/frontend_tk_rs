use serde::{Deserialize, Serialize};

use super::{PathItemParameters, ServerObject, operation_object::OperationObject};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PathItemObject {
    #[serde(rename = "$ref")]
    pub r#ref: Option<String>,

    pub summary: Option<String>,

    pub description: Option<String>,

    pub get: Option<OperationObject>,

    pub put: Option<OperationObject>,

    pub post: Option<OperationObject>,

    pub delete: Option<OperationObject>,

    pub options: Option<OperationObject>,

    pub head: Option<OperationObject>,

    pub patch: Option<OperationObject>,

    pub trace: Option<OperationObject>,

    pub servers: Option<Vec<ServerObject>>,

    pub parameters: Option<PathItemParameters>,
}
