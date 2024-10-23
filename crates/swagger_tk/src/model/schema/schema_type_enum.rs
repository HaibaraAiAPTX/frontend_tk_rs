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

impl ToString for SchemaTypeEnum {
    fn to_string(&self) -> String {
        match &self {
            SchemaTypeEnum::String => "string".to_string(),
            SchemaTypeEnum::Object => "object".to_string(),
            SchemaTypeEnum::Array => "array".to_string(),
            SchemaTypeEnum::Integer => "integer".to_string(),
            SchemaTypeEnum::Number => "number".to_string(),
            SchemaTypeEnum::Boolean => "boolean".to_string(),
        }
    }
}
