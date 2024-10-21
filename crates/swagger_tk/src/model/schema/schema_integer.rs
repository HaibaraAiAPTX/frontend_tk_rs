use serde::{Deserialize, Serialize};

use super::SchemaTypeEnum;

#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaInteger {
    pub r#type: SchemaTypeEnum,

    pub format: Option<String>,
}
