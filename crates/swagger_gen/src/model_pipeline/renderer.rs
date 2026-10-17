use std::collections::HashMap;

use crate::utils::format_ts_code;

use super::model::{
    ModelIr, ModelKind, ModelLiteral, ModelProperty, ModelRenderStyle, ModelType, NullableType,
    ScalarType,
};

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
                let rows = render_property_rows(properties, style, &model.name);
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

/// 渲染一组对象属性（接口体 / 内联对象体共用）。
fn render_property_rows(
    properties: &[ModelProperty],
    style: ModelRenderStyle,
    current_model_name: &str,
) -> String {
    properties
        .iter()
        .map(|property| {
            let description = property
                .description
                .as_ref()
                .map(|text| format!("\n/** {text} */\n"))
                .unwrap_or_default();
            let optional_symbol = if property.required { ":" } else { "?:" };
            let ts_type = render_type(
                &property.r#type,
                style,
                current_model_name,
                property.nullable,
            );
            format!("{description}{}{optional_symbol}{ts_type}", property.name)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// 渲染一个类型槽位（类型本体 + 自身可空）。
fn render_slot(slot: &NullableType, style: ModelRenderStyle, current_model_name: &str) -> String {
    render_type(&slot.r#type, style, current_model_name, slot.nullable)
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
        ModelType::Dictionary { value } => {
            let child = render_slot(value, style, current_model_name);
            format!("Record<string, {child}>")
        }
        ModelType::InlineObject {
            properties,
            additional_properties,
        } => {
            let rows = render_property_rows(properties, style, current_model_name);
            let literal = format!("{{ {rows} }}");
            match additional_properties {
                // `properties` + `additionalProperties: <schema>` 无法用单一声明表达，
                // 交叉类型是唯一保留两侧信息的写法。
                Some(index) => {
                    let index_type = render_slot(index, style, current_model_name);
                    format!("{literal} & Record<string, {index_type}>")
                }
                None => literal,
            }
        }
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

    fn prop(name: &str, r#type: ModelType, required: bool, nullable: bool) -> ModelProperty {
        ModelProperty {
            name: name.to_string(),
            description: None,
            required,
            nullable,
            r#type,
        }
    }

    fn slot(r#type: ModelType, nullable: bool) -> NullableType {
        NullableType {
            r#type: Box::new(r#type),
            nullable,
        }
    }

    fn render_interface(properties: Vec<ModelProperty>) -> String {
        render_single_model(ModelNode {
            name: "Sample".to_string(),
            description: None,
            kind: ModelKind::Interface { properties },
        })
    }

    /// 字典值 = 原始类型 / $ref / 数组 × 值可空 × 属性 required / 属性可空 的组合矩阵。
    #[test]
    fn renders_dictionary_value_type_matrix() {
        let cases = [
            (
                "stringRequired",
                slot(ModelType::Scalar(ScalarType::String), false),
                true,
                false,
                "stringRequired: Record<string, string>",
            ),
            (
                "stringOptional",
                slot(ModelType::Scalar(ScalarType::String), false),
                false,
                false,
                "stringOptional?: Record<string, string>",
            ),
            (
                "stringValueNullable",
                slot(ModelType::Scalar(ScalarType::String), true),
                false,
                false,
                "stringValueNullable?: Record<string, string | null>",
            ),
            (
                "integerValue",
                slot(
                    ModelType::Scalar(ScalarType::Integer(IntegerSpec {
                        format: Default::default(),
                    })),
                    false,
                ),
                true,
                false,
                "integerValue: Record<string, number>",
            ),
            (
                "numberValue",
                slot(
                    ModelType::Scalar(ScalarType::Number(NumberSpec {
                        format: Default::default(),
                    })),
                    false,
                ),
                true,
                false,
                "numberValue: Record<string, number>",
            ),
            (
                "booleanValue",
                slot(ModelType::Scalar(ScalarType::Boolean), false),
                true,
                false,
                "booleanValue: Record<string, boolean>",
            ),
            (
                "refValue",
                slot(
                    ModelType::Ref {
                        name: "Other".to_string(),
                    },
                    false,
                ),
                true,
                false,
                "refValue: Record<string, import(\"./Other\").Other>",
            ),
            (
                "arrayOfScalar",
                slot(
                    ModelType::Array {
                        item: Box::new(ModelType::Scalar(ScalarType::String)),
                    },
                    false,
                ),
                true,
                false,
                "arrayOfScalar: Record<string, Array<string>>",
            ),
            (
                "arrayOfRefNullable",
                slot(
                    ModelType::Array {
                        item: Box::new(ModelType::Ref {
                            name: "Other".to_string(),
                        }),
                    },
                    true,
                ),
                true,
                false,
                "arrayOfRefNullable: Record<string, Array<import(\"./Other\").Other> | null>",
            ),
            (
                "selfRefValue",
                slot(
                    ModelType::Ref {
                        name: "Sample".to_string(),
                    },
                    false,
                ),
                true,
                false,
                // 自引用必须输出裸名，不能写成 import("./Sample").Sample
                "selfRefValue: Record<string, Sample>",
            ),
        ];

        for (name, value, required, nullable, expected) in cases {
            let rendered = render_interface(vec![prop(
                name,
                ModelType::Dictionary { value },
                required,
                nullable,
            )]);
            assert!(
                rendered.contains(expected),
                "`{name}` should render as `{expected}`, got:\n{rendered}"
            );
        }
    }

    #[test]
    fn renders_dictionary_property_nullable_as_trailing_null() {
        let rendered = render_interface(vec![prop(
            "Data",
            ModelType::Dictionary {
                value: slot(ModelType::Scalar(ScalarType::String), true),
            },
            false,
            true,
        )]);

        assert!(
            rendered.contains("Data?: Record<string, string | null> | null"),
            "got:\n{rendered}"
        );
    }

    /// 回归护栏：`Object` 仍是自由对象，不得被字典改造波及。
    #[test]
    fn renders_free_form_object_as_bare_object() {
        let rendered = render_interface(vec![prop("freeForm", ModelType::Object, true, false)]);

        assert!(rendered.contains("freeForm: object"), "got:\n{rendered}");
    }

    #[test]
    fn renders_inline_object_with_optional_and_required_keys() {
        let rendered = render_interface(vec![prop(
            "fixed",
            ModelType::InlineObject {
                properties: vec![
                    prop(
                        "count",
                        ModelType::Scalar(ScalarType::Integer(IntegerSpec {
                            format: Default::default(),
                        })),
                        true,
                        false,
                    ),
                    prop("label", ModelType::Scalar(ScalarType::String), false, true),
                ],
                additional_properties: None,
            },
            false,
            true,
        )]);

        assert!(rendered.contains("fixed?:"), "got:\n{rendered}");
        assert!(rendered.contains("count: number"), "got:\n{rendered}");
        assert!(
            rendered.contains("label?: string | null"),
            "got:\n{rendered}"
        );
        assert!(!rendered.contains("object"), "got:\n{rendered}");
    }

    /// `properties` + `additionalProperties: <schema>` 的混合形态：交叉类型保留两侧信息。
    #[test]
    fn renders_mixed_object_as_intersection_with_record() {
        let rendered = render_interface(vec![prop(
            "mixed",
            ModelType::InlineObject {
                properties: vec![prop(
                    "count",
                    ModelType::Scalar(ScalarType::Integer(IntegerSpec {
                        format: Default::default(),
                    })),
                    true,
                    false,
                )],
                additional_properties: Some(slot(ModelType::Scalar(ScalarType::String), false)),
            },
            true,
            false,
        )]);

        assert!(rendered.contains("count: number"), "got:\n{rendered}");
        assert!(
            rendered.contains("Record<string, string>"),
            "got:\n{rendered}"
        );
        assert!(rendered.contains('&'), "got:\n{rendered}");
    }

    #[test]
    fn renders_top_level_dictionary_alias_as_record() {
        let rendered = render_single_model(ModelNode {
            name: "StringMap".to_string(),
            description: None,
            kind: ModelKind::Alias {
                target: ModelType::Dictionary {
                    value: slot(ModelType::Scalar(ScalarType::String), false),
                },
                nullable: false,
            },
        });

        assert!(
            rendered.contains("export type StringMap = Record<string, string>"),
            "got:\n{rendered}"
        );
    }
}
