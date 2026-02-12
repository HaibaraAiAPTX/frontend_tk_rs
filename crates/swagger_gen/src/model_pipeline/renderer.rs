use std::collections::HashMap;

use crate::utils::format_ts_code;

use super::model::{ModelIr, ModelKind, ModelLiteral, ModelRenderStyle, ModelType};

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
                            ModelLiteral::Number { value } => value.to_string(),
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
        ModelType::String => "string".to_string(),
        ModelType::Number => "number".to_string(),
        ModelType::Boolean => "boolean".to_string(),
        ModelType::Object => "object".to_string(),
        ModelType::Ref { name } => {
            if style == ModelRenderStyle::Module && name != current_model_name {
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
            ModelLiteral::Number { value } => value.to_string(),
        },
    };

    if nullable {
        format!("{base} | null")
    } else {
        base
    }
}
