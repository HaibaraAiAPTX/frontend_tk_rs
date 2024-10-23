use serde::{Deserialize, Serialize};

use super::SchemaEnum;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaTypeObject {
    pub schema: SchemaEnum,
}
