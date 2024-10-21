use serde::{Deserialize, Serialize};

use super::SchemaTypeEnum;

#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaBool {
    #[serde(rename = "type")]
    pub r#type: SchemaTypeEnum,
}
