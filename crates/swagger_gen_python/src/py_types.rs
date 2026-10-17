//! ModelType -> Python type string mapping and import collection.

use swagger_gen::model_pipeline::{
    ModelLiteral, ModelType, NullableType, NumberFormat, NumberSpec, ScalarType,
};

/// Render a ModelType to a Python type string.
pub fn render_python_type(model_type: &ModelType) -> String {
    let base = match model_type {
        ModelType::Scalar(scalar) => render_python_scalar_type(scalar),
        ModelType::String => "str".to_string(),
        ModelType::Number => "float".to_string(),
        ModelType::Boolean => "bool".to_string(),
        ModelType::Object => "dict[str, Any]".to_string(),
        ModelType::Ref { name } => name.to_string(),
        ModelType::Array { item } => {
            let inner = render_python_type(item);
            format!("list[{inner}]")
        }
        ModelType::Union { variants } => {
            let mut rendered_variants = Vec::new();
            for variant in variants {
                let rendered = render_python_type(variant);
                if !rendered_variants.contains(&rendered) {
                    rendered_variants.push(rendered);
                }
            }
            rendered_variants.join(" | ")
        }
        ModelType::Literal { value } => match value {
            ModelLiteral::String { value } => format!("Literal[\"{value}\"]"),
            ModelLiteral::Integer { value } => format!("Literal[{value}]"),
            ModelLiteral::Number {
                format: NumberFormat::Decimal,
                ..
            } => "Decimal".to_string(),
            ModelLiteral::Number { value, .. } => format!("Literal[{value}]"),
        },
        ModelType::Dictionary { value } => {
            let inner = render_python_slot(value);
            format!("dict[str, {inner}]")
        }
        // 内联固定键对象没有对应的 Pydantic 模型文件，Python 侧无法用单行类型字符串表达
        // 键集合；沿用历史行为渲染为 `dict[str, Any]`，保证 Python 产物零变化。
        ModelType::InlineObject { .. } => "dict[str, Any]".to_string(),
    };
    base
}

/// Render a type slot (type + its own nullability).
fn render_python_slot(slot: &NullableType) -> String {
    render_python_type_nullable(&slot.r#type, slot.nullable)
}

fn render_python_scalar_type(scalar: &ScalarType) -> String {
    match scalar {
        ScalarType::String => "str".to_string(),
        ScalarType::Boolean => "bool".to_string(),
        ScalarType::Integer(_) => "int".to_string(),
        ScalarType::Number(number) => match number.format {
            NumberFormat::Decimal => "Decimal".to_string(),
            NumberFormat::Float | NumberFormat::Double | NumberFormat::Unknown => {
                "float".to_string()
            }
        },
    }
}

/// Render a ModelType with nullable support.
pub fn render_python_type_nullable(model_type: &ModelType, nullable: bool) -> String {
    let base = render_python_type(model_type);
    if nullable {
        format!("{base} | None")
    } else {
        base
    }
}

/// Collect Python imports needed for a given ModelType.
pub fn collect_python_imports(model_type: &ModelType) -> Vec<String> {
    let mut imports = Vec::new();
    collect_imports_recursive(model_type, &mut imports);
    imports.sort();
    imports.dedup();
    imports
}

fn collect_imports_recursive(model_type: &ModelType, imports: &mut Vec<String>) {
    match model_type {
        ModelType::Scalar(scalar) => collect_imports_for_scalar(scalar, imports),
        ModelType::Object => {
            if !imports.contains(&"from typing import Any".to_string()) {
                imports.push("from typing import Any".to_string());
            }
        }
        ModelType::Array { item } => {
            collect_imports_recursive(item, imports);
        }
        ModelType::Union { variants } => {
            for v in variants {
                collect_imports_recursive(v, imports);
            }
        }
        // 字典值可能含 `Decimal` / `Literal` / `Any` / `$ref`，必须递归进去，
        // 否则 `dict[str, Decimal]` 会漏掉 `from decimal import Decimal`。
        ModelType::Dictionary { value } => {
            collect_imports_recursive(&value.r#type, imports);
        }
        ModelType::InlineObject {
            properties,
            additional_properties,
        } => {
            // 渲染结果是 `dict[str, Any]`，因此必须补 `Any`。
            if !imports.contains(&"from typing import Any".to_string()) {
                imports.push("from typing import Any".to_string());
            }
            for property in properties {
                collect_imports_recursive(&property.r#type, imports);
            }
            if let Some(index) = additional_properties {
                collect_imports_recursive(&index.r#type, imports);
            }
        }
        ModelType::Literal { value } => {
            collect_imports_for_literal(model_type, imports);
            if matches!(
                value,
                ModelLiteral::String { .. } | ModelLiteral::Integer { .. }
            ) && !imports.contains(&"from typing import Literal".to_string())
            {
                imports.push("from typing import Literal".to_string());
            }
        }
        _ => {}
    }
}

fn collect_imports_for_scalar(scalar: &ScalarType, imports: &mut Vec<String>) {
    if matches!(
        scalar,
        ScalarType::Number(NumberSpec {
            format: NumberFormat::Decimal
        })
    ) && !imports.contains(&"from decimal import Decimal".to_string())
    {
        imports.push("from decimal import Decimal".to_string());
    }
}

fn collect_imports_for_literal(model_type: &ModelType, imports: &mut Vec<String>) {
    if let ModelType::Literal {
        value:
            ModelLiteral::Number {
                format: NumberFormat::Decimal,
                ..
            },
    } = model_type
    {
        if !imports.contains(&"from decimal import Decimal".to_string()) {
            imports.push("from decimal import Decimal".to_string());
        }
    }
}

/// Convert a camelCase/PascalCase name to snake_case.
pub fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_type() {
        assert_eq!(render_python_type(&ModelType::String), "str");
    }

    #[test]
    fn test_scalar_integer_type() {
        assert_eq!(
            render_python_type(&ModelType::Scalar(ScalarType::Integer(
                swagger_gen::model_pipeline::IntegerSpec {
                    format: swagger_gen::model_pipeline::IntegerFormat::Unknown,
                },
            ))),
            "int"
        );
    }

    #[test]
    fn test_scalar_number_float_type() {
        assert_eq!(
            render_python_type(&ModelType::Scalar(ScalarType::Number(NumberSpec {
                format: NumberFormat::Float,
            },))),
            "float"
        );
    }

    #[test]
    fn test_scalar_number_decimal_type() {
        assert_eq!(
            render_python_type(&ModelType::Scalar(ScalarType::Number(NumberSpec {
                format: NumberFormat::Decimal,
            },))),
            "Decimal"
        );
    }

    #[test]
    fn test_boolean_type() {
        assert_eq!(render_python_type(&ModelType::Boolean), "bool");
    }

    #[test]
    fn test_object_type() {
        assert_eq!(render_python_type(&ModelType::Object), "dict[str, Any]");
    }

    #[test]
    fn test_ref_type() {
        assert_eq!(
            render_python_type(&ModelType::Ref {
                name: "User".to_string()
            }),
            "User"
        );
    }

    #[test]
    fn test_array_type() {
        assert_eq!(
            render_python_type(&ModelType::Array {
                item: Box::new(ModelType::String)
            }),
            "list[str]"
        );
    }

    #[test]
    fn test_union_type() {
        assert_eq!(
            render_python_type(&ModelType::Union {
                variants: vec![ModelType::String, ModelType::Number,]
            }),
            "str | float"
        );
    }

    #[test]
    fn test_union_deduplicates_decimal_literals() {
        assert_eq!(
            render_python_type(&ModelType::Union {
                variants: vec![
                    ModelType::Literal {
                        value: ModelLiteral::Number {
                            value: "1.25".to_string(),
                            format: NumberFormat::Decimal,
                        },
                    },
                    ModelType::Literal {
                        value: ModelLiteral::Number {
                            value: "2.50".to_string(),
                            format: NumberFormat::Decimal,
                        },
                    },
                ]
            }),
            "Decimal"
        );
    }

    #[test]
    fn test_literal_string() {
        assert_eq!(
            render_python_type(&ModelType::Literal {
                value: ModelLiteral::String {
                    value: "active".to_string()
                }
            }),
            "Literal[\"active\"]"
        );
    }

    #[test]
    fn test_literal_number() {
        assert_eq!(
            render_python_type(&ModelType::Literal {
                value: ModelLiteral::Number {
                    value: "42".to_string(),
                    format: swagger_gen::model_pipeline::NumberFormat::Unknown,
                }
            }),
            "Literal[42]"
        );
    }

    #[test]
    fn test_literal_decimal_type() {
        assert_eq!(
            render_python_type(&ModelType::Literal {
                value: ModelLiteral::Number {
                    value: "1.25".to_string(),
                    format: NumberFormat::Decimal,
                }
            }),
            "Decimal"
        );
    }

    #[test]
    fn test_nullable() {
        assert_eq!(
            render_python_type_nullable(&ModelType::String, true),
            "str | None"
        );
        assert_eq!(
            render_python_type_nullable(&ModelType::String, false),
            "str"
        );
    }

    #[test]
    fn test_imports_object() {
        let imports = collect_python_imports(&ModelType::Object);
        assert!(imports.contains(&"from typing import Any".to_string()));
    }

    #[test]
    fn test_imports_decimal_scalar() {
        let imports = collect_python_imports(&ModelType::Scalar(ScalarType::Number(NumberSpec {
            format: NumberFormat::Decimal,
        })));
        assert!(imports.contains(&"from decimal import Decimal".to_string()));
    }

    #[test]
    fn test_imports_decimal_literal() {
        let imports = collect_python_imports(&ModelType::Literal {
            value: ModelLiteral::Number {
                value: "1.25".to_string(),
                format: NumberFormat::Decimal,
            },
        });
        assert!(imports.contains(&"from decimal import Decimal".to_string()));
        assert!(!imports.contains(&"from typing import Literal".to_string()));
    }

    #[test]
    fn test_imports_literal() {
        let imports = collect_python_imports(&ModelType::Literal {
            value: ModelLiteral::String {
                value: "x".to_string(),
            },
        });
        assert!(imports.contains(&"from typing import Literal".to_string()));
    }

    #[test]
    fn test_imports_array_nested_object() {
        let imports = collect_python_imports(&ModelType::Array {
            item: Box::new(ModelType::Object),
        });
        assert!(imports.contains(&"from typing import Any".to_string()));
    }

    fn slot(r#type: ModelType, nullable: bool) -> NullableType {
        NullableType {
            r#type: Box::new(r#type),
            nullable,
        }
    }

    #[test]
    fn test_dictionary_renders_as_dict_of_value_type() {
        assert_eq!(
            render_python_type(&ModelType::Dictionary {
                value: slot(ModelType::Scalar(ScalarType::String), false),
            }),
            "dict[str, str]"
        );
        assert_eq!(
            render_python_type(&ModelType::Dictionary {
                value: slot(ModelType::Scalar(ScalarType::String), true),
            }),
            "dict[str, str | None]"
        );
        assert_eq!(
            render_python_type(&ModelType::Dictionary {
                value: slot(ModelType::Scalar(ScalarType::Boolean), false),
            }),
            "dict[str, bool]"
        );
        assert_eq!(
            render_python_type(&ModelType::Dictionary {
                value: slot(
                    ModelType::Ref {
                        name: "Other".to_string()
                    },
                    false
                ),
            }),
            "dict[str, Other]"
        );
        assert_eq!(
            render_python_type(&ModelType::Dictionary {
                value: slot(
                    ModelType::Array {
                        item: Box::new(ModelType::Ref {
                            name: "Other".to_string()
                        })
                    },
                    true
                ),
            }),
            "dict[str, list[Other] | None]"
        );
    }

    #[test]
    fn test_dictionary_decimal_value_collects_decimal_import() {
        let imports = collect_python_imports(&ModelType::Dictionary {
            value: slot(
                ModelType::Scalar(ScalarType::Number(NumberSpec {
                    format: NumberFormat::Decimal,
                })),
                false,
            ),
        });
        assert!(
            imports.contains(&"from decimal import Decimal".to_string()),
            "dictionary value must recurse into import collection, got: {imports:?}"
        );
    }

    #[test]
    fn test_inline_object_keeps_legacy_dict_any_rendering() {
        let inline = ModelType::InlineObject {
            properties: vec![swagger_gen::model_pipeline::ModelProperty {
                name: "count".to_string(),
                description: None,
                required: true,
                nullable: false,
                r#type: ModelType::Scalar(ScalarType::Integer(
                    swagger_gen::model_pipeline::IntegerSpec {
                        format: swagger_gen::model_pipeline::IntegerFormat::Unknown,
                    },
                )),
            }],
            additional_properties: None,
        };
        assert_eq!(render_python_type(&inline), "dict[str, Any]");
        assert!(collect_python_imports(&inline).contains(&"from typing import Any".to_string()));
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("camelCase"), "camel_case");
        assert_eq!(to_snake_case("PascalCase"), "pascal_case");
        assert_eq!(to_snake_case("already_snake"), "already_snake");
        assert_eq!(to_snake_case("HTTPSUrl"), "h_t_t_p_s_url");
    }
}
