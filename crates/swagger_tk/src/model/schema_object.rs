use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SchemaObject {
  #[serde(rename = "$ref")]
  pub r#ref: Option<String>,

  #[serde(rename = "type")]
  pub r#type: Option<String>,

  pub required: Option<Vec<String>>,

  pub format: Option<String>,

  pub default: Option<String>,

  #[serde(rename = "enum")]
  pub r#enum: Option<Vec<String>>,

  pub items: Option<Box<SchemaObject>>,

  #[serde(rename = "additionalProperties")]
  pub additional_properties: Option<Box<SchemaObject>>
}
