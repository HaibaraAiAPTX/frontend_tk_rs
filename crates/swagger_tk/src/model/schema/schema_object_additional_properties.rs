use serde::{Deserialize, Serialize};

use super::SchemaEnum;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SchemaObjectAdditionalProperties {
    Boolean(bool),
    Schema(Box<SchemaEnum>),
}
