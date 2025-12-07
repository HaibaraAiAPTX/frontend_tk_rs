use serde::{Deserialize, Serialize};

use super::{PathItemObject, ReferenceObject};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum CallbackValue {
    PathItem(Box<PathItemObject>),
    Reference(ReferenceObject),
}
