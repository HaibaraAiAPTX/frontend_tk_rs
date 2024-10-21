use serde::{Deserialize, Serialize};

use super::SchemaEnum;

#[derive(Debug,Serialize,Deserialize)]
pub struct MediaTypeObject {
  pub schema: SchemaEnum
}