use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaStringFormat {
    Date,
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

impl SchemaStringFormat {
    fn as_str(&self) -> &str {
        match self {
            Self::Date => "date",
            Self::DateTime => "date-time",
            Self::Password => "password",
            Self::Byte => "byte",
            Self::Binary => "binary",
            Self::Email => "email",
            Self::Uuid => "uuid",
            Self::Uri => "uri",
            Self::Hostname => "hostname",
            Self::Ipv4 => "ipv4",
            Self::Ipv6 => "ipv6",
            Self::Other(value) => value.as_str(),
        }
    }
}

impl Serialize for SchemaStringFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SchemaStringFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        let format = match raw.as_str() {
            "date" => Self::Date,
            "date-time" => Self::DateTime,
            "password" => Self::Password,
            "byte" => Self::Byte,
            "binary" => Self::Binary,
            "email" => Self::Email,
            "uuid" => Self::Uuid,
            "uri" => Self::Uri,
            "hostname" => Self::Hostname,
            "ipv4" => Self::Ipv4,
            "ipv6" => Self::Ipv6,
            _ => Self::Other(raw),
        };
        Ok(format)
    }
}

#[cfg(test)]
mod tests {
    use super::SchemaStringFormat;

    #[test]
    fn should_parse_known_format() {
        let parsed: SchemaStringFormat =
            serde_json::from_str("\"date-time\"").expect("parse date-time format should pass");
        assert_eq!(parsed, SchemaStringFormat::DateTime);
    }

    #[test]
    fn should_parse_unknown_format_to_other() {
        let parsed: SchemaStringFormat =
            serde_json::from_str("\"tel\"").expect("parse tel format should pass");
        assert_eq!(parsed, SchemaStringFormat::Other("tel".to_string()));
    }
}
