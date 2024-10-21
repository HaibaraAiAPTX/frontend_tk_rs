use serde::{Deserialize, Serialize};

use super::{SchemaObject, SchemaString};
use crate::model::ReferenceObject;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SchemaEnum {
    Ref(ReferenceObject),
    String(SchemaString),
    Object(SchemaObject),
}
