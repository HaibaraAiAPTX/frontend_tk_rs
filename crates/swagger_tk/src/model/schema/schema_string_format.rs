use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SchemaStringFormat {
    Date,
    #[serde(rename = "date-time")]
    DateTime,
    Password,
    Byte,
    Binary,
    Email,
    Uuid,
    Uri,
    Hostname,
    Ipv4,
    Ipv6,
    Other(String),
}
