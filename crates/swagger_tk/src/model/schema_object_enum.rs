use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SchemaObjectEnum {
    Boolean(bool),
    String(String),
    Number(i32),
}
