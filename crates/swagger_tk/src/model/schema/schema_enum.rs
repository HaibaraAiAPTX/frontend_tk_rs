use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;

use super::{SchemaArray, SchemaBool, SchemaInteger, SchemaNumber, SchemaObject, SchemaString};
use crate::model::ReferenceObject;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum SchemaEnum {
    Ref(ReferenceObject),
    Object(SchemaObject),
    String(SchemaString),
    Integer(SchemaInteger),
    Number(SchemaNumber),
    Boolean(SchemaBool),
    Array(SchemaArray),
}

impl<'de> Deserialize<'de> for SchemaEnum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Value::deserialize(deserializer)?;
        let r#ref = v.get("$ref");
        if r#ref.is_some() {
            let a = ReferenceObject::deserialize(v).map_err(de::Error::custom)?;
            return Ok(SchemaEnum::Ref(a));
        }

        let r#type = v
            .get("type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| de::Error::missing_field("\"type\" field is missing"))?;

        match r#type {
            "string" => Ok(SchemaEnum::String(
                SchemaString::deserialize(v).map_err(de::Error::custom)?,
            )),
            "object" => Ok(SchemaEnum::Object(
                SchemaObject::deserialize(v).map_err(de::Error::custom)?,
            )),
            "number" => Ok(SchemaEnum::Number(
                SchemaNumber::deserialize(v).map_err(de::Error::custom)?,
            )),
            "integer" => Ok(SchemaEnum::Integer(
                SchemaInteger::deserialize(v).map_err(de::Error::custom)?,
            )),
            "boolean" => Ok(SchemaEnum::Boolean(
                SchemaBool::deserialize(v).map_err(de::Error::custom)?,
            )),
            "array" => Ok(SchemaEnum::Array(
                SchemaArray::deserialize(v).map_err(de::Error::custom)?,
            )),
            _ => Err(de::Error::missing_field("Unsupported model conversions")),
        }
    }
}
