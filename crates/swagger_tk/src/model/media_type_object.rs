use serde::{Deserialize, Serialize};

use super::SchemaObject;

#[derive(Debug,Serialize,Deserialize)]
pub struct MediaTypeObject {
  pub schema: SchemaObject
}