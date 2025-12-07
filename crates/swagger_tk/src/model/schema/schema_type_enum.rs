use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SchemaTypeEnum {
    String,
    Object,
    Array,
    Integer,
    Number,
    Boolean,
}

impl Display for SchemaTypeEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            SchemaTypeEnum::String => write!(f, "string"),
            SchemaTypeEnum::Object => write!(f, "object"),
            SchemaTypeEnum::Array => write!(f, "array"),
            SchemaTypeEnum::Integer => write!(f, "integer"),
            SchemaTypeEnum::Number => write!(f, "number"),
            SchemaTypeEnum::Boolean => write!(f, "boolean"),
        }
    }
}
