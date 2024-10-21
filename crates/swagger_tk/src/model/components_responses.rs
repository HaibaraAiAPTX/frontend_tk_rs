use serde::{Deserialize, Serialize};

use super::{ReferenceObject, ResponseObject};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ComponentsResponses {
    Response(ResponseObject),
    Reference(ReferenceObject),
}
