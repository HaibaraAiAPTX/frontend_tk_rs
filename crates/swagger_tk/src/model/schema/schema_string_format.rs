use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemaStringFormat {
    Date,
    #[serde(rename = "date-time")]
    DateTime,
    Password,
    Byte,
    Binay,
    Email,
    Uuid,
    Uri,
    Hostname,
    Ipv4,
    Ipv6,
    Other(String),
}
