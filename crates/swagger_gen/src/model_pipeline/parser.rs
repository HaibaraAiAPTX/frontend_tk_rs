use swagger_tk::{
    getter::get_all_schema,
    model::{OpenAPIObject, SchemaEnum},
};

use crate::utils::{ReferenceObjectExtension, SchemaEnumExtension};

use super::model::{
    IntegerFormat, IntegerSpec, ModelEnumMember, ModelIr, ModelKind, ModelLiteral, ModelNode,
    ModelProperty, ModelType, NumberFormat, NumberSpec, ScalarType,
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
                let required = v.required.as_ref();
                let mut properties = v
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
                ModelKind::Interface { properties }
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
        SchemaEnum::Object(_) => ModelType::Object,
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
}
