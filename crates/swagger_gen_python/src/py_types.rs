//! ModelType -> Python type string mapping and import collection.

use swagger_gen::model_pipeline::{ModelLiteral, ModelType};

/// Render a ModelType to a Python type string.
pub fn render_python_type(model_type: &ModelType) -> String {
    let base = match model_type {
        ModelType::String => "str".to_string(),
        ModelType::Number => "float".to_string(),
        ModelType::Boolean => "bool".to_string(),
        ModelType::Object => "dict[str, Any]".to_string(),
        ModelType::Ref { name } => name.to_string(),
        ModelType::Array { item } => {
            let inner = render_python_type(item);
            format!("list[{inner}]")
        }
        ModelType::Union { variants } => variants
            .iter()
            .map(|v| render_python_type(v))
            .collect::<Vec<_>>()
            .join(" | "),
        ModelType::Literal { value } => match value {
            ModelLiteral::String { value } => format!("Literal[\"{value}\"]"),
            ModelLiteral::Number { value } => format!("Literal[{value}]"),
        },
    };
    base
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
        ModelType::Literal { .. } => {
            if !imports.contains(&"from typing import Literal".to_string()) {
                imports.push("from typing import Literal".to_string());
            }
        }
        _ => {}
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
    fn test_number_type() {
        assert_eq!(render_python_type(&ModelType::Number), "float");
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
                variants: vec![
                    ModelType::String,
                    ModelType::Number,
                ]
            }),
            "str | float"
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
                    value: "42".to_string()
                }
            }),
            "Literal[42]"
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

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("camelCase"), "camel_case");
        assert_eq!(to_snake_case("PascalCase"), "pascal_case");
        assert_eq!(to_snake_case("already_snake"), "already_snake");
        assert_eq!(to_snake_case("HTTPSUrl"), "h_t_t_p_s_url");
    }
}
