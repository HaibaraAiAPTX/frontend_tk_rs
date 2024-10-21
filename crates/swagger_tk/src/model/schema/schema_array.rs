use serde::{self, Deserialize, Serialize};

use super::{SchemaEnum, SchemaTypeEnum};

#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaArray {
    #[serde(rename = "type")]
    pub r#type: SchemaTypeEnum,

    pub items: Option<Box<SchemaEnum>>,
}
