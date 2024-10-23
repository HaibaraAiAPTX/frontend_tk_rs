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
