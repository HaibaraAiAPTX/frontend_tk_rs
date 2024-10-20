use serde::{Deserialize, Serialize};

use super::{ReferenceObject, ResponseObject};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponsesValue {
    Response(ResponseObject),
    Reference(ReferenceObject),
}
