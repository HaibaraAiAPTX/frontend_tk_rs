use std::collections::HashMap;

use crate::utils::format_ts_code;

use super::model::{ModelIr, ModelKind, ModelLiteral, ModelRenderStyle, ModelType, ScalarType};

pub fn render_model_files(
    ir: &ModelIr,
    style: ModelRenderStyle,
    only_names: &[String],
) -> Result<HashMap<String, String>, String> {
    let name_filter = if only_names.is_empty() {
        None
    } else {
        Some(
            only_names
                .iter()
                .cloned()
                .collect::<std::collections::HashSet<_>>(),
        )
    };

    let mut files = HashMap::new();
    for model in &ir.models {
        if let Some(filter) = name_filter.as_ref() {
            if !filter.contains(&model.name) {
                continue;
            }
        }

        let source = match &model.kind {
            ModelKind::Interface { properties } => {
                let rows = properties
                    .iter()
                    .map(|property| {
                        let description = property
                            .description
                            .as_ref()
                            .map(|text| format!("\n/** {text} */\n"))
                            .unwrap_or_default();
                        let optional_symbol = if property.required { ":" } else { "?:" };
                        let ts_type =
                            render_type(&property.r#type, style, &model.name, property.nullable);
                        format!("{description}{}{optional_symbol}{ts_type}", property.name)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let description = model
                    .description
                    .as_ref()
                    .map(|text| format!("/** {text} */\n"))
                    .unwrap_or_default();
                match style {
                    ModelRenderStyle::Declaration => {
                        format!("{description}declare interface {} {{{rows}}}", model.name)
                    }
                    ModelRenderStyle::Module => {
                        format!("{description}export interface {} {{{rows}}}", model.name)
                    }
                }
            }
            ModelKind::Enum { members } => {
                let enum_rows = members
                    .iter()
                    .map(|member| {
                        let value_text = match &member.value {
                            ModelLiteral::String { value } => serde_json::to_string(value)
                                .unwrap_or_else(|_| format!("\"{value}\"")),
                            ModelLiteral::Integer { value } => value.to_string(),
                            ModelLiteral::Number { value, .. } => value.to_string(),
                        };
                        let comment = member
                            .comment
                            .as_ref()
                            .map(|text| format!("/** {text} */\n"))
                            .unwrap_or_default();
                        format!("{comment}{} = {}", member.name, value_text)
                    })
                    .collect::<Vec<_>>()
                    .join(",\n");
                let description = model
                    .description
                    .as_ref()
                    .map(|text| format!("/** {text} */\n"))
                    .unwrap_or_default();
                format!(
                    "{description}export enum {} {{\n{}\n}}",
                    model.name, enum_rows
                )
            }
            ModelKind::Alias { target, nullable } => {
                let ts_type = render_type(target, style, &model.name, *nullable);
                let description = model
                    .description
                    .as_ref()
                    .map(|text| format!("/** {text} */\n"))
                    .unwrap_or_default();
                match style {
                    ModelRenderStyle::Declaration => {
                        format!("{description}declare type {} = {ts_type}", model.name)
                    }
                    ModelRenderStyle::Module => {
                        format!("{description}export type {} = {ts_type}", model.name)
                    }
                }
            }
        };

        let file_name = match style {
            ModelRenderStyle::Declaration => match model.kind {
                ModelKind::Enum { .. } => format!("{}.ts", model.name),
                _ => format!("{}.d.ts", model.name),
            },
            ModelRenderStyle::Module => format!("{}.ts", model.name),
        };
        files.insert(file_name, format_ts_code(&source)?);
    }
    Ok(files)
}

fn render_type(
    model_type: &ModelType,
    style: ModelRenderStyle,
    current_model_name: &str,
    nullable: bool,
) -> String {
    let base = match model_type {
        ModelType::Scalar(ScalarType::String) => "string".to_string(),
        ModelType::Scalar(ScalarType::Boolean) => "boolean".to_string(),
        ModelType::Scalar(ScalarType::Integer(_)) => "number".to_string(),
        ModelType::Scalar(ScalarType::Number(_)) => "number".to_string(),
        ModelType::String => "string".to_string(),
        ModelType::Number => "number".to_string(),
        ModelType::Boolean => "boolean".to_string(),
        ModelType::Object => "object".to_string(),
        ModelType::Ref { name } => {
            // Both Module and Declaration styles need import for cross-references
            if name != current_model_name {
                // Use dynamic import for both Module and Declaration styles
                format!("import(\"./{}\").{}", name, name)
            } else {
                name.to_string()
            }
        }
        ModelType::Array { item } => {
            let child = render_type(item, style, current_model_name, false);
            format!("Array<{child}>")
        }
        ModelType::Union { variants } => variants
            .iter()
            .map(|item| render_type(item, style, current_model_name, false))
            .collect::<Vec<_>>()
            .join(" | "),
        ModelType::Literal { value } => match value {
            ModelLiteral::String { value } => {
                serde_json::to_string(value).unwrap_or_else(|_| format!("\"{value}\""))
            }
            ModelLiteral::Integer { value } => value.to_string(),
            ModelLiteral::Number { value, .. } => value.to_string(),
        },
    };

    if nullable {
        format!("{base} | null")
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::super::model::{
        IntegerSpec, ModelEnumMember, ModelNode, ModelProperty, NumberFormat, NumberSpec,
    };
    use super::*;

    fn render_single_model(model: ModelNode) -> String {
        let files = render_model_files(
            &ModelIr {
                models: vec![model],
            },
            ModelRenderStyle::Module,
            &[],
        )
        .expect("render model files fail");

        files
            .values()
            .next()
            .cloned()
            .expect("model file should be rendered")
    }

    #[test]
    fn renders_integer_and_number_scalars_as_number_in_typescript() {
        let rendered = render_single_model(ModelNode {
            name: "Sample".to_string(),
            description: None,
            kind: ModelKind::Interface {
                properties: vec![
                    ModelProperty {
                        name: "intValue".to_string(),
                        description: None,
                        required: true,
                        nullable: false,
                        r#type: ModelType::Scalar(ScalarType::Integer(IntegerSpec {
                            format: Default::default(),
                        })),
                    },
                    ModelProperty {
                        name: "numValue".to_string(),
                        description: None,
                        required: true,
                        nullable: false,
                        r#type: ModelType::Scalar(ScalarType::Number(NumberSpec {
                            format: Default::default(),
                        })),
                    },
                ],
            },
        });

        assert!(rendered.contains("intValue: number"));
        assert!(rendered.contains("numValue: number"));
        assert!(!rendered.contains("intValue: integer"));
        assert!(!rendered.contains("numValue: double"));
    }

    #[test]
    fn renders_string_and_boolean_scalars_as_expected_types_in_typescript() {
        let rendered = render_single_model(ModelNode {
            name: "ScalarKinds".to_string(),
            description: None,
            kind: ModelKind::Interface {
                properties: vec![
                    ModelProperty {
                        name: "textValue".to_string(),
                        description: None,
                        required: true,
                        nullable: false,
                        r#type: ModelType::Scalar(ScalarType::String),
                    },
                    ModelProperty {
                        name: "flagValue".to_string(),
                        description: None,
                        required: true,
                        nullable: false,
                        r#type: ModelType::Scalar(ScalarType::Boolean),
                    },
                ],
            },
        });

        assert!(rendered.contains("textValue: string"));
        assert!(rendered.contains("flagValue: boolean"));
    }

    #[test]
    fn renders_numeric_literals_as_ts_numeric_literals() {
        let rendered = render_single_model(ModelNode {
            name: "NumericLiterals".to_string(),
            description: None,
            kind: ModelKind::Enum {
                members: vec![
                    ModelEnumMember {
                        name: "IntLiteral".to_string(),
                        value: ModelLiteral::Integer {
                            value: "42".to_string(),
                        },
                        comment: None,
                    },
                    ModelEnumMember {
                        name: "NumLiteral".to_string(),
                        value: ModelLiteral::Number {
                            value: "3.14".to_string(),
                            format: NumberFormat::Unknown,
                        },
                        comment: None,
                    },
                ],
            },
        });

        assert!(rendered.contains("IntLiteral = 42"));
        assert!(rendered.contains("NumLiteral = 3.14"));
    }
}
