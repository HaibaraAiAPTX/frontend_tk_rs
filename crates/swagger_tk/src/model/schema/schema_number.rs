use serde::{Deserialize, Serialize};

use super::SchemaTypeEnum;

#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaNumber {
    pub r#type: SchemaTypeEnum,

    pub format: Option<String>,

    pub description: Option<String>,
}
