use super::{PathItemObject, ReferenceObject};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ComponentsPathItems {
    PathItem(Box<PathItemObject>),
    Reference(ReferenceObject),
}
