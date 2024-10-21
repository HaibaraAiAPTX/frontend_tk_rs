use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemaTypeEnum {
    String,
    Object,
    Array,
    Integer,
    Number,
    Boolean,
}
