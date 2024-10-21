use super::{HeaderObject, ReferenceObject};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ComponentsHeaders {
    Header(HeaderObject),
    Reference(ReferenceObject),
}
