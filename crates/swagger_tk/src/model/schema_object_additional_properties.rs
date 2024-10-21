use serde::{Deserialize, Serialize};

use super::SchemaEnum;

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SchemaObjectAdditionalProperties {
    Boolean(bool),
    Schema(SchemaEnum),
}
