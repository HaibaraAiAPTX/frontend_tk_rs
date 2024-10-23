use serde::{Deserialize, Serialize};

use super::{LinkObject, ReferenceObject};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ResponseLink {
    Link(LinkObject),
    Reference(ReferenceObject),
}
