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
        //
        // 非字符串的 `type`（3.1 的联合数组、显式 `null`）不参与推断：本 crate 的
        // `SchemaTypeEnum` 是单值枚举，把联合类型硬塞进单一变体只会得到错误的模型，
        // 所以这里给出可定位的明确错误，而不是静默降级。
        let r#type = match v.get("type") {
            Some(Value::String(name)) => SchemaTypeEnum::deserialize(Value::String(name.clone()))
                .map_err(de::Error::custom)?,
            Some(other) => {
                return Err(de::Error::custom(format!(
                    "`type` {other} is neither a string nor absent; this crate maps a schema to a \
                     single `SchemaTypeEnum`, so OpenAPI 3.1 type unions (`type: [...]`) and an \
                     explicit `type: null` must be normalized, or the schema declared with one \
                     concrete `type`"
                )));
            }
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
    // 显式字符串 `type` 已在调用方校验过，这里只需处理「缺失」这一种情况。
    if value.get("type").is_none() {
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

/// 由 `enum` 成员推断类型。
///
/// 拒绝空数组与含 `null` 的成员：本 crate 的标量变体把 `enum` 存成 `Vec<String>` /
/// `Vec<i32>` / `Vec<f32>`，既表达不了空枚举，也表达不了可空成员。与其推断成功、随后在
/// 变体反序列化时抛出「invalid type: null」这种与推断无关的错误，不如在这里一次说清。
fn infer_enum_type(members: &[Value]) -> Result<SchemaTypeEnum, String> {
    if members.is_empty() {
        return Err(
            "`type` is missing and `enum` is empty; declare `type` explicitly (an empty `enum` \
             cannot be represented by this crate's scalar variants)"
                .to_string(),
        );
    }

    let Some(first) = members.iter().find(|member| !member.is_null()) else {
        return Err(format!(
            "`type` is missing and `enum` {members:?} contains only `null`; this crate's scalar \
             variants cannot hold `null` members, declare `type` explicitly or drop the `null`"
        ));
    };
    if members.iter().any(Value::is_null) {
        return Err(format!(
            "`type` is missing and `enum` {members:?} contains `null`; this crate's scalar \
             variants cannot hold `null` members, declare `type` explicitly or drop the `null`"
        ));
    }

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
        Value::Null => unreachable!("null members are rejected above"),
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
    let mut names = Vec::new();
    for member in members {
        let name = match member {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(number) if is_integer_number(number) => "integer",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        };
        // 去重要跨位置生效：`["a", 1, "b"]` 应打印 `string / integer`，
        // 只去相邻重复会漏掉这种交替出现的形态。
        if !names.contains(&name) {
            names.push(name);
        }
    }
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
