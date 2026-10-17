use serde::{Deserialize, Deserializer, Serialize, de};
use serde_json::{Map, Value};

use super::{
    SchemaArray, SchemaBool, SchemaInteger, SchemaNumber, SchemaObject, SchemaString,
    SchemaTypeEnum,
};
use crate::model::ReferenceObject;

#[derive(Debug, Serialize, Clone)]
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

        if v.get("$ref").is_some() {
            let a = ReferenceObject::deserialize(v).map_err(de::Error::custom)?;
            return Ok(SchemaEnum::Ref(a));
        }

        // `type` 缺失在 OpenAPI 3.0 / 3.1 里是合法写法（例如 `additionalProperties: {}`），
        // 此时按结构关键字推断；没有任何线索才落成自由 object。
        // `type: null` 与 3.1 的 `type: [...]` 也走推断，避免整份 spec 解析失败。
        let r#type = match v.get("type").and_then(|t| t.as_str()) {
            Some(name) => SchemaTypeEnum::deserialize(Value::String(name.to_string()))
                .map_err(de::Error::custom)?,
            None => infer_type_from_shape(&v).map_err(de::Error::custom)?,
        };

        // 完全空 schema（`{}`）语义等价于「任意值」，即 `additionalProperties: true`；
        // 显式记下来，避免下游把 `additional_properties == None` 读成「封闭对象」。
        let is_any = v.as_object().is_some_and(Map::is_empty);

        match r#type {
            SchemaTypeEnum::String => Ok(SchemaEnum::String(
                deserialize_with_type(v, &r#type, is_any, SchemaString::deserialize)
                    .map_err(de::Error::custom)?,
            )),
            SchemaTypeEnum::Object => Ok(SchemaEnum::Object(
                deserialize_with_type(v, &r#type, is_any, SchemaObject::deserialize)
                    .map_err(de::Error::custom)?,
            )),
            SchemaTypeEnum::Number => Ok(SchemaEnum::Number(
                deserialize_with_type(v, &r#type, is_any, SchemaNumber::deserialize)
                    .map_err(de::Error::custom)?,
            )),
            SchemaTypeEnum::Integer => Ok(SchemaEnum::Integer(
                deserialize_with_type(v, &r#type, is_any, SchemaInteger::deserialize)
                    .map_err(de::Error::custom)?,
            )),
            SchemaTypeEnum::Boolean => Ok(SchemaEnum::Boolean(
                deserialize_with_type(v, &r#type, is_any, SchemaBool::deserialize)
                    .map_err(de::Error::custom)?,
            )),
            SchemaTypeEnum::Array => Ok(SchemaEnum::Array(
                deserialize_with_type(v, &r#type, is_any, SchemaArray::deserialize)
                    .map_err(de::Error::custom)?,
            )),
        }
    }
}

/// 用推断出的 `type` 补齐 JSON 后，再走各变体原有的 derive 反序列化。
///
/// 只在 `type` 缺失时补写，因此显式 `type` 分支的行为与改造前完全一致；变体内记录的
/// `type` 也始终来自输入或推断结果，不依赖 serde 默认值。
fn deserialize_with_type<T, F>(
    mut value: Value,
    r#type: &SchemaTypeEnum,
    is_any: bool,
    deserialize: F,
) -> Result<T, serde_json::Error>
where
    F: FnOnce(Value) -> Result<T, serde_json::Error>,
{
    if !value.get("type").is_some_and(Value::is_string) {
        let object = value.as_object_mut().ok_or_else(|| {
            serde::de::Error::custom(format!(
                "schema with inferred `type` {} must be a JSON object",
                r#type
            ))
        })?;
        object.insert("type".to_string(), Value::String(r#type.to_string()));
        if is_any {
            object.insert("additionalProperties".to_string(), Value::Bool(true));
        }
    }
    deserialize(value)
}

/// 无 `type` 时的推断规则（按顺序取第一条命中的线索）：
///
/// 1. `items` / `uniqueItems` / `minItems` / `maxItems` → array
/// 2. `properties` / `additionalProperties` → object
/// 3. `enum` → 按首个非 `null` 成员推断
/// 4. 其余（含空 schema `{}`）→ object（自由对象，等价于 `additionalProperties: true`）
///
/// 刻意不纳入 `format` / `maxLength` / `minimum` 这类跨类型通用关键字：它们无法唯一
/// 决定类型，纳入只会让推断结果变得不可预期。
fn infer_type_from_shape(value: &Value) -> Result<SchemaTypeEnum, String> {
    const ARRAY_HINTS: [&str; 4] = ["items", "uniqueItems", "minItems", "maxItems"];
    const OBJECT_HINTS: [&str; 2] = ["properties", "additionalProperties"];

    if ARRAY_HINTS.iter().any(|key| value.get(key).is_some()) {
        return Ok(SchemaTypeEnum::Array);
    }
    if OBJECT_HINTS.iter().any(|key| value.get(key).is_some()) {
        return Ok(SchemaTypeEnum::Object);
    }
    if let Some(Value::Array(members)) = value.get("enum") {
        return infer_enum_type(members);
    }
    Ok(SchemaTypeEnum::Object)
}

/// 由 `enum` 成员推断类型；空数组与全 `null` 视为 string。
fn infer_enum_type(members: &[Value]) -> Result<SchemaTypeEnum, String> {
    let Some(first) = members.iter().find(|member| !member.is_null()) else {
        // `enum: []` 没有约束力，落到 string，与显式 `type: string` 的既有行为一致。
        return Ok(SchemaTypeEnum::String);
    };

    let inferred = match first {
        Value::String(_) => SchemaTypeEnum::String,
        Value::Number(number) if is_integer_number(number) => SchemaTypeEnum::Integer,
        Value::Number(_) => SchemaTypeEnum::Number,
        Value::Bool(_) => SchemaTypeEnum::Boolean,
        Value::Object(_) | Value::Array(_) => {
            return Err(format!(
                "`type` is missing and `enum` member {first} is neither string / integer / \
                 number / boolean"
            ));
        }
        Value::Null => unreachable!("null members are skipped by `find`"),
    };

    if !members
        .iter()
        .all(|member| matches_enum_type(member, &inferred))
    {
        return Err(format!(
            "`type` is missing and `enum` mixes {}: {members:?}",
            describe_json_types(members)
        ));
    }
    Ok(inferred)
}

/// `enum` 成员是否与推断出的类型相容（`null` 视为可空，一并接受）。
fn matches_enum_type(member: &Value, r#type: &SchemaTypeEnum) -> bool {
    match member {
        Value::Null => true,
        Value::String(_) => *r#type == SchemaTypeEnum::String,
        Value::Bool(_) => *r#type == SchemaTypeEnum::Boolean,
        Value::Number(number) => match r#type {
            SchemaTypeEnum::Integer => is_integer_number(number),
            SchemaTypeEnum::Number => true,
            _ => false,
        },
        Value::Array(_) | Value::Object(_) => false,
    }
}

fn is_integer_number(number: &serde_json::Number) -> bool {
    number.is_i64() || number.is_u64()
}

fn describe_json_types(members: &[Value]) -> String {
    let mut names = members
        .iter()
        .map(|member| match member {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(number) if is_integer_number(number) => "integer",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        })
        .collect::<Vec<_>>();
    names.dedup();
    names.join(" / ")
}

impl SchemaEnum {
    /// 获取描述
    pub fn get_description(&self) -> Option<&String> {
        match self {
            SchemaEnum::Ref(v) => v.description.as_ref(),
            SchemaEnum::Object(v) => v.description.as_ref(),
            SchemaEnum::String(v) => v.description.as_ref(),
            SchemaEnum::Integer(v) => v.description.as_ref(),
            SchemaEnum::Number(v) => v.description.as_ref(),
            SchemaEnum::Boolean(v) => v.description.as_ref(),
            SchemaEnum::Array(v) => v.description.as_ref(),
        }
    }
}
