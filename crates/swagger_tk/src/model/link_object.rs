use serde::{Deserialize, Serialize};

use super::ServerObject;

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkObject {
  #[serde(rename = "operationRef")]
  pub operation_ref: Option<String>,

  #[serde(rename = "operationId")]
  pub operation_id: Option<String>,

  pub description: Option<String>,

  pub server: Option<ServerObject>
}