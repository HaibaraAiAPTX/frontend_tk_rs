use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{SchemaObjectAdditionalProperties, SchemaObjectEnum};

/// 数据结构描述对象  
/// TODO：这个对象太乱了，实际上的情况是根据不同的 type 会有不同的属性，又或者只有 $ref 属性，后期有时间就改一下
#[derive(Debug, Deserialize, Serialize)]
pub struct SchemaObject {
  #[serde(rename = "$ref")]
  pub r#ref: Option<String>,

  pub description: Option<String>,

  /// 是否可空
  pub nullable: Option<bool>,

  #[serde(rename = "type")]
  pub r#type: Option<String>,

  #[serde(rename = "minItems")]
  pub min_items: Option<u32>,

  /// 属性列表
  pub properties: Option<HashMap<String, SchemaObject>>,

  pub required: Option<Vec<String>>,

  pub format: Option<String>,

  pub default: Option<String>,

  #[serde(rename = "enum")]
  pub r#enum: Option<Vec<SchemaObjectEnum>>,

  pub items: Option<Box<SchemaObject>>,

  #[serde(rename = "additionalProperties")]
  pub additional_properties: Option<Box<SchemaObjectAdditionalProperties>>
}
