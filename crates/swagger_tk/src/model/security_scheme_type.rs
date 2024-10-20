use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SecuritySchemeType {
    #[serde(rename = "apiKey")]
    ApiKey,
    #[serde(rename = "http")]
    Http,
    #[serde(rename = "mutualTLS")]
    MutualTLS,
    #[serde(rename = "oauth2")]
    OAuth2,
    #[serde(rename = "openIdConnect")]
    OpenIDConnect,
}
