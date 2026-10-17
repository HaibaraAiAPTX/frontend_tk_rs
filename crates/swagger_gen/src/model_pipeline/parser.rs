use swagger_tk::{
    getter::get_all_schema,
    model::{OpenAPIObject, SchemaEnum, SchemaObject, SchemaObjectAdditionalProperties},
};

use crate::utils::{ReferenceObjectExtension, SchemaEnumExtension};

use super::model::{
    IntegerFormat, IntegerSpec, ModelEnumMember, ModelIr, ModelKind, ModelLiteral, ModelNode,
    ModelProperty, ModelType, NullableType, NumberFormat, NumberSpec, ScalarType,
};

pub fn build_model_ir(open_api: &OpenAPIObject) -> Result<ModelIr, String> {
    let schemas = get_all_schema(open_api).ok_or("get all schema fail")?;
    let mut schema_names = schemas.keys().cloned().collect::<Vec<_>>();
    schema_names.sort();

    let mut models = Vec::new();
    for name in schema_names {
        let schema = schemas
            .get(&name)
            .ok_or_else(|| format!("can't find {name} schema"))?;
        models.push(schema_to_model_node(&name, schema, open_api));
    }
    Ok(ModelIr { models })
}

fn schema_to_model_node(name: &str, schema: &SchemaEnum, open_api: &OpenAPIObject) -> ModelNode {
    ModelNode {
        name: name.to_string(),
        description: schema.get_description().cloned(),
        kind: match schema {
            SchemaEnum::Object(v) => {
                let properties = object_properties(v, open_api);
                if properties.is_empty() {
                    // 顶层纯字典：`{ type: object, additionalProperties: <schema> }`。
                    // 不能退化成空 interface，否则键值类型被静默吞掉。
                    match dictionary_value_type(v, open_api) {
                        Some(value) => ModelKind::Alias {
                            target: ModelType::Dictionary { value },
                            nullable: v.nullable.unwrap_or(false),
                        },
                        None => ModelKind::Interface { properties },
                    }
                } else {
                    ModelKind::Interface { properties }
                }
            }
            SchemaEnum::String(v) => {
                if let Some(items) = v.r#enum.as_ref() {
                    ModelKind::Enum {
                        members: items
                            .iter()
                            .enumerate()
                            .map(|(index, value)| ModelEnumMember {
                                name: format!("Value{}", index + 1),
                                value: ModelLiteral::String {
                                    value: value.to_string(),
                                },
                                comment: None,
                            })
                            .collect::<Vec<_>>(),
                    }
                } else {
                    ModelKind::Alias {
                        target: string_model_type(),
                        nullable: v.nullable.unwrap_or(false),
                    }
                }
            }
            SchemaEnum::Integer(v) => {
                if let Some(items) = v.r#enum.as_ref() {
                    ModelKind::Enum {
                        members: integer_enum_members(items),
                    }
                } else {
                    ModelKind::Alias {
                        target: integer_model_type(v.format.as_deref()),
                        nullable: v.nullable.unwrap_or(false),
                    }
                }
            }
            SchemaEnum::Number(v) => {
                if let Some(items) = v.r#enum.as_ref() {
                    ModelKind::Enum {
                        members: number_enum_members(items, v.format.as_deref()),
                    }
                } else {
                    ModelKind::Alias {
                        target: number_model_type(v.format.as_deref()),
                        nullable: v.nullable.unwrap_or(false),
                    }
                }
            }
            _ => ModelKind::Alias {
                target: schema_to_model_type(schema, open_api),
                nullable: schema.can_be_null(open_api),
            },
        },
    }
}

fn schema_to_model_type(schema: &SchemaEnum, open_api: &OpenAPIObject) -> ModelType {
    match schema {
        SchemaEnum::Ref(reference) => ModelType::Ref {
            name: reference.get_type_name(),
        },
        SchemaEnum::Object(v) => object_model_type(v, open_api),
        SchemaEnum::String(v) => v
            .r#enum
            .as_ref()
            .map(|values| ModelType::Union {
                variants: values
                    .iter()
                    .map(|value| string_literal_model_type(value))
                    .collect::<Vec<_>>(),
            })
            .unwrap_or_else(string_model_type),
        SchemaEnum::Integer(v) => v
            .r#enum
            .as_ref()
            .map(|values| ModelType::Union {
                variants: values
                    .iter()
                    .map(integer_literal_model_type)
                    .collect::<Vec<_>>(),
            })
            .unwrap_or_else(|| integer_model_type(v.format.as_deref())),
        SchemaEnum::Number(v) => v
            .r#enum
            .as_ref()
            .map(|values| {
                let number_format = parse_number_format(v.format.as_deref());
                ModelType::Union {
                    variants: values
                        .iter()
                        .map(|value| number_literal_model_type(value, number_format.clone()))
                        .collect::<Vec<_>>(),
                }
            })
            .unwrap_or_else(|| number_model_type(v.format.as_deref())),
        SchemaEnum::Boolean(_) => boolean_model_type(),
        SchemaEnum::Array(v) => ModelType::Array {
            item: Box::new(schema_to_model_type(&v.items, open_api)),
        },
    }
}

/// `type: object` 的类型映射：
///
/// - 有 `properties` → `InlineObject`（保留键、required、nullable、描述）
/// - 无 `properties` 且有 `additionalProperties: <schema>` → `Dictionary`
/// - 其余（`additionalProperties: true` / 省略）→ `Object`（自由对象，行为与旧版一致）
fn object_model_type(schema: &SchemaObject, open_api: &OpenAPIObject) -> ModelType {
    let properties = object_properties(schema, open_api);
    let additional_properties = dictionary_value_type(schema, open_api);

    if !properties.is_empty() {
        return ModelType::InlineObject {
            properties,
            additional_properties,
        };
    }

    match additional_properties {
        Some(value) => ModelType::Dictionary { value },
        None => ModelType::Object,
    }
}

/// 解析 `properties` 为按名称排序的 `ModelProperty` 列表（空 `properties` 视同缺失）。
fn object_properties(schema: &SchemaObject, open_api: &OpenAPIObject) -> Vec<ModelProperty> {
    let required = schema.required.as_ref();
    let mut properties = schema
        .properties
        .as_ref()
        .map(|p| {
            let mut keys = p.keys().collect::<Vec<_>>();
            keys.sort();
            keys.into_iter()
                .map(|key| {
                    let child = p.get(key).expect("schema key must exist");
                    ModelProperty {
                        name: key.to_string(),
                        description: child.get_description().cloned(),
                        required: required.is_some_and(|items| items.contains(key)),
                        nullable: child.can_be_null(open_api),
                        r#type: schema_to_model_type(child, open_api),
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    properties.sort_by(|a, b| a.name.cmp(&b.name));
    properties
}

/// 从 `additionalProperties` 提取字典值类型；`true` / `false` / 缺失一律返回 `None`。
fn dictionary_value_type(schema: &SchemaObject, open_api: &OpenAPIObject) -> Option<NullableType> {
    match schema.additional_properties.as_ref()? {
        SchemaObjectAdditionalProperties::Boolean(_) => None,
        SchemaObjectAdditionalProperties::Schema(value) => Some(NullableType {
            r#type: Box::new(schema_to_model_type(value, open_api)),
            nullable: value.can_be_null(open_api),
        }),
    }
}

fn string_model_type() -> ModelType {
    ModelType::Scalar(ScalarType::String)
}

fn boolean_model_type() -> ModelType {
    ModelType::Scalar(ScalarType::Boolean)
}

fn integer_model_type(format: Option<&str>) -> ModelType {
    ModelType::Scalar(ScalarType::Integer(IntegerSpec {
        format: parse_integer_format(format),
    }))
}

fn number_model_type(format: Option<&str>) -> ModelType {
    ModelType::Scalar(ScalarType::Number(NumberSpec {
        format: parse_number_format(format),
    }))
}

fn string_literal_model_type(value: &str) -> ModelType {
    ModelType::Literal {
        value: ModelLiteral::String {
            value: value.to_string(),
        },
    }
}

fn integer_literal_model_type(value: &i32) -> ModelType {
    ModelType::Literal {
        value: ModelLiteral::Integer {
            value: value.to_string(),
        },
    }
}

fn number_literal_model_type(value: &f32, format: NumberFormat) -> ModelType {
    ModelType::Literal {
        value: ModelLiteral::Number {
            value: value.to_string(),
            format,
        },
    }
}

fn integer_enum_members(values: &[i32]) -> Vec<ModelEnumMember> {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| ModelEnumMember {
            name: format!("Value{}", index + 1),
            value: ModelLiteral::Integer {
                value: value.to_string(),
            },
            comment: None,
        })
        .collect::<Vec<_>>()
}

fn number_enum_members(values: &[f32], format: Option<&str>) -> Vec<ModelEnumMember> {
    let number_format = parse_number_format(format);
    values
        .iter()
        .enumerate()
        .map(|(index, value)| ModelEnumMember {
            name: format!("Value{}", index + 1),
            value: ModelLiteral::Number {
                value: value.to_string(),
                format: number_format.clone(),
            },
            comment: None,
        })
        .collect::<Vec<_>>()
}

fn parse_integer_format(format: Option<&str>) -> IntegerFormat {
    match format {
        Some("int32") => IntegerFormat::Int32,
        Some("int64") => IntegerFormat::Int64,
        _ => IntegerFormat::Unknown,
    }
}

fn parse_number_format(format: Option<&str>) -> NumberFormat {
    match format {
        Some("float") => NumberFormat::Float,
        Some("double") => NumberFormat::Double,
        Some("decimal") => NumberFormat::Decimal,
        _ => NumberFormat::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    use crate::model_pipeline::{
        IntegerFormat, IntegerSpec, ModelLiteral, ModelType, NumberFormat, NumberSpec, ScalarType,
    };
    use swagger_tk::model::OpenAPIObject;

    #[test]
    fn parses_numeric_formats_into_distinct_scalar_types() {
        let open_api = OpenAPIObject::from_str(
            r#"
{
  "openapi": "3.1.0",
  "info": { "title": "numeric-test", "version": "1.0.0" },
  "paths": {},
  "components": {
    "schemas": {
      "Ratio": { "type": "number", "format": "double" },
      "WholeCount": { "type": "integer", "format": "int64" }
    }
  }
}
"#,
        )
        .expect("parse openapi object fail");

        let ir = build_model_ir(&open_api).expect("build model ir fail");

        let whole_count = ir
            .models
            .iter()
            .find(|model| model.name == "WholeCount")
            .expect("WholeCount model should exist");
        assert!(matches!(
            &whole_count.kind,
            ModelKind::Alias {
                target: ModelType::Scalar(ScalarType::Integer(IntegerSpec {
                    format: IntegerFormat::Int64
                })),
                nullable: false
            }
        ));

        let ratio = ir
            .models
            .iter()
            .find(|model| model.name == "Ratio")
            .expect("Ratio model should exist");
        assert!(matches!(
            &ratio.kind,
            ModelKind::Alias {
                target: ModelType::Scalar(ScalarType::Number(NumberSpec {
                    format: NumberFormat::Double
                })),
                nullable: false
            }
        ));
    }

    #[test]
    fn parses_numeric_formats_missing_format_as_unknown() {
        let open_api = OpenAPIObject::from_str(
            r#"
{
  "openapi": "3.1.0",
  "info": { "title": "numeric-missing-format-test", "version": "1.0.0" },
  "paths": {},
  "components": {
    "schemas": {
      "PlainCount": { "type": "integer" },
      "PlainRatio": { "type": "number" }
    }
  }
}
"#,
        )
        .expect("parse openapi object fail");

        let ir = build_model_ir(&open_api).expect("build model ir fail");

        let plain_count = ir
            .models
            .iter()
            .find(|model| model.name == "PlainCount")
            .expect("PlainCount model should exist");
        assert!(matches!(
            &plain_count.kind,
            ModelKind::Alias {
                target: ModelType::Scalar(ScalarType::Integer(IntegerSpec {
                    format: IntegerFormat::Unknown
                })),
                nullable: false
            }
        ));

        let plain_ratio = ir
            .models
            .iter()
            .find(|model| model.name == "PlainRatio")
            .expect("PlainRatio model should exist");
        assert!(matches!(
            &plain_ratio.kind,
            ModelKind::Alias {
                target: ModelType::Scalar(ScalarType::Number(NumberSpec {
                    format: NumberFormat::Unknown
                })),
                nullable: false
            }
        ));
    }

    #[test]
    fn parses_numeric_enums_into_distinct_literal_kinds() {
        let open_api = OpenAPIObject::from_str(
            r#"
{
  "openapi": "3.1.0",
  "info": { "title": "numeric-enum-test", "version": "1.0.0" },
  "paths": {},
  "components": {
    "schemas": {
      "RatioStates": { "type": "number", "enum": [1.5, 2.5] },
      "WholeStates": { "type": "integer", "enum": [1, 2] }
    }
  }
}
"#,
        )
        .expect("parse openapi object fail");

        let ir = build_model_ir(&open_api).expect("build model ir fail");

        let whole_states = ir
            .models
            .iter()
            .find(|model| model.name == "WholeStates")
            .expect("WholeStates model should exist");
        let ModelKind::Enum { members } = &whole_states.kind else {
            panic!("WholeStates should be an enum");
        };
        assert!(matches!(
            &members[0].value,
            ModelLiteral::Integer { value } if value == "1"
        ));
        assert!(matches!(
            &members[1].value,
            ModelLiteral::Integer { value } if value == "2"
        ));

        let ratio_states = ir
            .models
            .iter()
            .find(|model| model.name == "RatioStates")
            .expect("RatioStates model should exist");
        let ModelKind::Enum { members } = &ratio_states.kind else {
            panic!("RatioStates should be an enum");
        };
        assert!(matches!(
            &members[0].value,
            ModelLiteral::Number {
                value,
                format: NumberFormat::Unknown
            } if value == "1.5"
        ));
        assert!(matches!(
            &members[1].value,
            ModelLiteral::Number {
                value,
                format: NumberFormat::Unknown
            } if value == "2.5"
        ));
    }

    #[test]
    fn parses_numeric_schemas_in_nested_properties_and_array_items() {
        let open_api = OpenAPIObject::from_str(
            r#"
{
  "openapi": "3.1.0",
  "info": { "title": "nested-numeric-test", "version": "1.0.0" },
  "paths": {},
  "components": {
    "schemas": {
      "Envelope": {
        "type": "object",
        "properties": {
          "count": { "type": "integer", "format": "int32" },
          "counts": {
            "type": "array",
            "items": { "type": "integer", "enum": [1, 2] }
          },
          "ratio": { "type": "number", "format": "double" },
          "ratios": {
            "type": "array",
            "items": { "type": "number", "enum": [1.5, 2.5], "format": "double" }
          }
        }
      }
    }
  }
}
"#,
        )
        .expect("parse openapi object fail");

        let ir = build_model_ir(&open_api).expect("build model ir fail");

        let envelope = ir
            .models
            .iter()
            .find(|model| model.name == "Envelope")
            .expect("Envelope model should exist");
        let ModelKind::Interface { properties } = &envelope.kind else {
            panic!("Envelope should be an interface");
        };

        let count = properties
            .iter()
            .find(|property| property.name == "count")
            .expect("count property should exist");
        assert!(matches!(
            &count.r#type,
            ModelType::Scalar(ScalarType::Integer(IntegerSpec {
                format: IntegerFormat::Int32
            }))
        ));

        let ratio = properties
            .iter()
            .find(|property| property.name == "ratio")
            .expect("ratio property should exist");
        assert!(matches!(
            &ratio.r#type,
            ModelType::Scalar(ScalarType::Number(NumberSpec {
                format: NumberFormat::Double
            }))
        ));

        let counts = properties
            .iter()
            .find(|property| property.name == "counts")
            .expect("counts property should exist");
        assert!(matches!(
            &counts.r#type,
            ModelType::Array { item }
                if matches!(
                    item.as_ref(),
                    ModelType::Union { variants }
                        if matches!(
                            &variants[0],
                            ModelType::Literal {
                                value: ModelLiteral::Integer { value }
                            } if value == "1"
                        ) && matches!(
                            &variants[1],
                            ModelType::Literal {
                                value: ModelLiteral::Integer { value }
                            } if value == "2"
                        )
                )
        ));

        let ratios = properties
            .iter()
            .find(|property| property.name == "ratios")
            .expect("ratios property should exist");
        assert!(matches!(
            &ratios.r#type,
            ModelType::Array { item }
                if matches!(
                    item.as_ref(),
                    ModelType::Union { variants }
                        if matches!(
                            &variants[0],
                            ModelType::Literal {
                                value: ModelLiteral::Number {
                                    value,
                                    format: NumberFormat::Double
                                }
                            } if value == "1.5"
                        ) && matches!(
                            &variants[1],
                            ModelType::Literal {
                                value: ModelLiteral::Number {
                                    value,
                                    format: NumberFormat::Double
                                }
                            } if value == "2.5"
                        )
                )
        ));
    }

    fn find_property<'a>(ir: &'a ModelIr, model: &str, property: &str) -> &'a ModelProperty {
        let node = ir
            .models
            .iter()
            .find(|item| item.name == model)
            .unwrap_or_else(|| panic!("{model} model should exist"));
        let ModelKind::Interface { properties } = &node.kind else {
            panic!("{model} should be an interface");
        };
        properties
            .iter()
            .find(|item| item.name == property)
            .unwrap_or_else(|| panic!("{model}.{property} should exist"))
    }

    fn parse_models(schemas: &str) -> ModelIr {
        let open_api = OpenAPIObject::from_str(&format!(
            r#"{{"openapi":"3.1.0","info":{{"title":"dict-test","version":"1.0.0"}},"paths":{{}},"components":{{"schemas":{schemas}}}}}"#
        ))
        .expect("parse openapi object fail");
        build_model_ir(&open_api).expect("build model ir fail")
    }

    #[test]
    fn parses_additional_properties_into_dictionary_kind() {
        let ir = parse_models(
            r##"{
              "Holder": {
                "type": "object",
                "properties": {
                  "textMap": { "type": "object", "additionalProperties": { "type": "string" } },
                  "nullableValueMap": {
                    "type": "object",
                    "additionalProperties": { "type": "string", "nullable": true }
                  },
                  "refMap": {
                    "type": "object",
                    "additionalProperties": { "$ref": "#/components/schemas/Other" }
                  },
                  "refArrayMap": {
                    "type": "object",
                    "additionalProperties": {
                      "type": "array",
                      "items": { "$ref": "#/components/schemas/Other" }
                    }
                  }
                }
              },
              "Other": { "type": "object", "properties": { "id": { "type": "string" } } }
            }"##,
        );

        assert!(matches!(
            &find_property(&ir, "Holder", "textMap").r#type,
            ModelType::Dictionary { value }
                if !value.nullable
                    && matches!(
                        value.r#type.as_ref(),
                        ModelType::Scalar(ScalarType::String)
                    )
        ));

        assert!(matches!(
            &find_property(&ir, "Holder", "nullableValueMap").r#type,
            ModelType::Dictionary { value }
                if value.nullable
                    && matches!(
                        value.r#type.as_ref(),
                        ModelType::Scalar(ScalarType::String)
                    )
        ));

        assert!(matches!(
            &find_property(&ir, "Holder", "refMap").r#type,
            ModelType::Dictionary { value }
                if !value.nullable
                    && matches!(
                        value.r#type.as_ref(),
                        ModelType::Ref { name } if name == "Other"
                    )
        ));

        assert!(matches!(
            &find_property(&ir, "Holder", "refArrayMap").r#type,
            ModelType::Dictionary { value }
                if matches!(
                    value.r#type.as_ref(),
                    ModelType::Array { item }
                        if matches!(
                            item.as_ref(),
                            ModelType::Ref { name } if name == "Other"
                        )
                )
        ));
    }

    #[test]
    fn parses_nested_object_properties_into_inline_object() {
        let ir = parse_models(
            r##"{
              "Holder": {
                "type": "object",
                "properties": {
                  "fixed": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                      "count": { "type": "integer", "format": "int32" },
                      "label": { "type": "string", "nullable": true }
                    },
                    "required": ["count"]
                  }
                }
              }
            }"##,
        );

        let ModelType::InlineObject {
            properties,
            additional_properties,
        } = &find_property(&ir, "Holder", "fixed").r#type
        else {
            panic!("fixed should parse as InlineObject");
        };
        assert!(additional_properties.is_none());
        assert_eq!(properties.len(), 2);

        let count = properties
            .iter()
            .find(|item| item.name == "count")
            .expect("count should exist");
        assert!(count.required);
        assert!(!count.nullable);
        assert!(matches!(
            count.r#type,
            ModelType::Scalar(ScalarType::Integer(IntegerSpec {
                format: IntegerFormat::Int32
            }))
        ));

        let label = properties
            .iter()
            .find(|item| item.name == "label")
            .expect("label should exist");
        assert!(!label.required);
        assert!(label.nullable);
    }

    #[test]
    fn parses_properties_plus_additional_properties_as_mixed_inline_object() {
        let ir = parse_models(
            r##"{
              "Holder": {
                "type": "object",
                "properties": {
                  "mixed": {
                    "type": "object",
                    "properties": { "count": { "type": "integer" } },
                    "additionalProperties": { "type": "string" }
                  }
                }
              }
            }"##,
        );

        let ModelType::InlineObject {
            properties,
            additional_properties,
        } = &find_property(&ir, "Holder", "mixed").r#type
        else {
            panic!("mixed should parse as InlineObject");
        };
        assert_eq!(properties.len(), 1);
        let index = additional_properties
            .as_ref()
            .expect("index signature should be preserved");
        assert!(!index.nullable);
        assert!(matches!(
            index.r#type.as_ref(),
            ModelType::Scalar(ScalarType::String)
        ));
    }

    #[test]
    fn parses_free_form_object_as_object_kind() {
        let ir = parse_models(
            r##"{
              "Holder": {
                "type": "object",
                "properties": {
                  "closed": { "type": "object", "additionalProperties": false },
                  "open": { "type": "object", "additionalProperties": true },
                  "bare": { "type": "object" }
                }
              }
            }"##,
        );

        for name in ["closed", "open", "bare"] {
            assert!(
                matches!(
                    &find_property(&ir, "Holder", name).r#type,
                    ModelType::Object
                ),
                "`{name}` should stay ModelType::Object"
            );
        }
    }

    #[test]
    fn parses_top_level_dictionary_schema_as_alias() {
        let ir = parse_models(
            r##"{
              "StringMap": {
                "type": "object",
                "additionalProperties": { "type": "string" }
              }
            }"##,
        );

        let model = ir
            .models
            .iter()
            .find(|item| item.name == "StringMap")
            .expect("StringMap model should exist");
        assert!(
            matches!(
                &model.kind,
                ModelKind::Alias {
                    target: ModelType::Dictionary { value },
                    nullable: false
                } if matches!(
                    value.r#type.as_ref(),
                    ModelType::Scalar(ScalarType::String)
                )
            ),
            "top-level dictionary must not degrade into an empty interface, got: {:?}",
            model.kind
        );
    }
}
