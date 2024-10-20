use serde::{Deserialize, Serialize};

use super::{OAuthFlowsObject, SecuritySchemeIn, SecuritySchemeType};

#[derive(Debug, Serialize, Deserialize)]
pub struct SecuritySchemaObject {
    #[serde(rename = "type")]
    pub r#type: SecuritySchemeType,

    pub description: Option<String>,

    pub name: String,

    #[serde(rename = "in")]
    pub r#in: SecuritySchemeIn,

    pub scheme: String,

    #[serde(rename = "bearerFormat")]
    pub bearer_format: Option<String>,

    pub flows: OAuthFlowsObject,

    #[serde(rename = "openIdConnectUrl")]
    pub open_id_connect_url: String,
}
