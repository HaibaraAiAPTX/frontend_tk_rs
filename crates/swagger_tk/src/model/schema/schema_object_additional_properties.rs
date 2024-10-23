use serde::{Deserialize, Serialize};

use super::SchemaEnum;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum SchemaObjectAdditionalProperties {
    Boolean(bool),
    Schema(Box<SchemaEnum>),
}
