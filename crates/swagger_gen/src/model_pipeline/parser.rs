use swagger_tk::{
    getter::get_all_schema,
    model::{OpenAPIObject, SchemaEnum},
};

use crate::utils::{ReferenceObjectExtension, SchemaEnumExtension};

use super::model::{
    ModelEnumMember, ModelIr, ModelKind, ModelLiteral, ModelNode, ModelProperty, ModelType,
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
                        keys
                            .into_iter()
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
                            })
                            .collect::<Vec<_>>(),
                    }
                } else {
                    ModelKind::Alias {
                        target: ModelType::String,
                        nullable: v.nullable.unwrap_or(false),
                    }
                }
            }
            SchemaEnum::Integer(v) => {
                if let Some(items) = v.r#enum.as_ref() {
                    ModelKind::Enum {
                        members: items
                            .iter()
                            .enumerate()
                            .map(|(index, value)| ModelEnumMember {
                                name: format!("Value{}", index + 1),
                                value: ModelLiteral::Number {
                                    value: value.to_string(),
                                },
                            })
                            .collect::<Vec<_>>(),
                    }
                } else {
                    ModelKind::Alias {
                        target: ModelType::Number,
                        nullable: v.nullable.unwrap_or(false),
                    }
                }
            }
            SchemaEnum::Number(v) => {
                if let Some(items) = v.r#enum.as_ref() {
                    ModelKind::Enum {
                        members: items
                            .iter()
                            .enumerate()
                            .map(|(index, value)| ModelEnumMember {
                                name: format!("Value{}", index + 1),
                                value: ModelLiteral::Number {
                                    value: value.to_string(),
                                },
                            })
                            .collect::<Vec<_>>(),
                    }
                } else {
                    ModelKind::Alias {
                        target: ModelType::Number,
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
            .map(|values| {
                ModelType::Union {
                    variants: values
                        .iter()
                        .map(|value| ModelType::Literal {
                            value: ModelLiteral::String {
                                value: value.to_string(),
                            },
                        })
                        .collect::<Vec<_>>(),
                }
            })
            .unwrap_or(ModelType::String),
        SchemaEnum::Integer(v) => v
            .r#enum
            .as_ref()
            .map(|values| {
                ModelType::Union {
                    variants: values
                        .iter()
                        .map(|value| ModelType::Literal {
                            value: ModelLiteral::Number {
                                value: value.to_string(),
                            },
                        })
                        .collect::<Vec<_>>(),
                }
            })
            .unwrap_or(ModelType::Number),
        SchemaEnum::Number(v) => v
            .r#enum
            .as_ref()
            .map(|values| {
                ModelType::Union {
                    variants: values
                        .iter()
                        .map(|value| ModelType::Literal {
                            value: ModelLiteral::Number {
                                value: value.to_string(),
                            },
                        })
                        .collect::<Vec<_>>(),
                }
            })
            .unwrap_or(ModelType::Number),
        SchemaEnum::Boolean(_) => ModelType::Boolean,
        SchemaEnum::Array(v) => ModelType::Array {
            item: Box::new(schema_to_model_type(&v.items, open_api)),
        },
    }
}

