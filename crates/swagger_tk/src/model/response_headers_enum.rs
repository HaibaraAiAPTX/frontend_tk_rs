use serde::{Deserialize, Serialize};

use super::{HeaderObject, ReferenceObject};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ResponseHeaders {
    Header(HeaderObject),
    Reference(ReferenceObject),
}
