use serde::{Deserialize, Serialize};

use super::OAuthFlowObject;

#[derive(Debug, Deserialize, Serialize)]
pub struct OAuthFlowsObject {
    pub implicit: Option<OAuthFlowObject>,

    pub password: Option<OAuthFlowObject>,

    #[serde(rename = "clientCredentials")]
    pub client_credentials: Option<OAuthFlowObject>,

    #[serde(rename = "authorizationCode")]
    pub authorization_code: Option<OAuthFlowObject>,
}
