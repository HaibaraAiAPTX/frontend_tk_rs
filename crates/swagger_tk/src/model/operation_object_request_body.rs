use serde::{Deserialize, Serialize};

use super::{ReferenceObject, RequestBodyObject};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum OperationObjectRequestBody {
    RequestBody(RequestBodyObject),
    Reference(ReferenceObject),
}
