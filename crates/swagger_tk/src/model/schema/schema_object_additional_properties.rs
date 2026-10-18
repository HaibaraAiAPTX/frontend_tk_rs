use serde::{Deserialize, Deserializer, Serialize, de};
use serde_json::Value;

use super::SchemaEnum;

/// `additionalProperties` 的取值：`bool` 或一段 schema。
///
/// **这里曾经是 crate 里唯一会吞掉 schema 报错的位置**，改动前请先读完下面两段：
///
/// - `#[serde(untagged)]` 的 derive 会把输入先缓冲成 `Content`，缓冲区**不保留内层
///   `D::Error`**，于是 `SchemaEnum::deserialize` 里那些精心写的诊断文案（「非字符串 `type`」
///   「`SchemaBool` 没有 `enum` 字段」等）走到这一层会被整体替换成
///   `data did not match any variant of untagged enum SchemaObjectAdditionalProperties`。
///   因此这里**手写 `Deserialize`**：先按 `Value` 收下输入，再按 JSON 类型分派到
///   `bool` 或 `SchemaEnum`，内层错误用 `de::Error::custom` 带出。
/// - 手写实现能成立的前提是**先把输入读成 `Value`**，而不是把 `deserializer` 直接交给
///   `SchemaEnum`：`SchemaEnum` 的错误类型是 `D::Error`，而 `D::Error` 可能是借用型的
///   `&str`，没法把内层错误取出来改写。`Value` 是自有的，转换没有这个问题。
///
/// 配合 `OpenAPIObject::from_str` 的 `serde_path_to_error`，非法 schema 的报错现在长这样：
///
/// ```text
/// components.schemas.Order: `additionalProperties`: `type` ["string","null"] is neither a
/// string nor absent; ...
/// ```
///
/// 字段名由本模块补（`serde_path_to_error` 只能追到宿主结构体，追不到具体字段），
/// 所属 schema 名由 `serde_path_to_error` 补。
#[derive(Debug, Serialize, Clone)]
pub enum SchemaObjectAdditionalProperties {
    Boolean(bool),
    Schema(Box<SchemaEnum>),
}

impl<'de> Deserialize<'de> for SchemaObjectAdditionalProperties {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        // `true` / `false` 是 OpenAPI 里最常用的写法，先分派掉；
        // 剩下的**必须**是 schema 对象，其它 JSON 类型直接报错而不是继续猜。
        match value {
            Value::Bool(flag) => Ok(SchemaObjectAdditionalProperties::Boolean(flag)),
            Value::Object(_) => SchemaEnum::deserialize(value)
                .map(|schema| SchemaObjectAdditionalProperties::Schema(Box::new(schema)))
                // 补上字段名：`serde_path_to_error` 只能把路径追到宿主结构体
                // （`components.schemas.Order`），追不到「哪个字段」，
                // 因为这里的错误是在字段值反序列化完成后才构造的。
                .map_err(|err| de::Error::custom(format!("`additionalProperties`: {err}"))),
            other => Err(de::Error::custom(format!(
                "`additionalProperties` must be a boolean or a schema object, got {}",
                describe_json_type(&other)
            ))),
        }
    }
}

/// 给「`additionalProperties` 取值非法」的报错用的 JSON 类型名。
fn describe_json_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
