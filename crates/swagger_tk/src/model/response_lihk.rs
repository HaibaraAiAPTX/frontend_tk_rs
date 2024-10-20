use serde::{Deserialize, Serialize};

use super::{LinkObject, ReferenceObject};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseLink {
    Link(LinkObject),
    Reference(ReferenceObject),
}
