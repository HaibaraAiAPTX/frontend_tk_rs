//! Type utilities for code generation.
//!
//! These utilities handle type normalization, validation, and import generation.

/// Normalizes a type reference to a valid TypeScript type.
///
/// - Returns primitive types as-is
/// - Returns valid identifier types as-is
/// - Returns "unknown" for invalid or complex types
pub fn normalize_type_ref(type_name: &str) -> String {
    let trimmed = type_name.trim();
    if is_primitive_type(trimmed) {
        return trimmed.to_string();
    }

    if is_identifier_type(trimmed) {
        return trimmed.to_string();
    }

    if !trimmed.is_empty() {
        return trimmed.to_string();
    }

    "unknown".to_string()
}

/// Checks if a string is a valid TypeScript identifier.
///
/// A valid identifier:
/// - Starts with a letter or underscore
/// - Contains only alphanumeric characters or underscores
pub fn is_identifier_type(type_name: &str) -> bool {
    let mut chars = type_name.chars();
    let first = chars.next();
    first.is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

/// Checks if a type name is a TypeScript primitive type.
pub fn is_primitive_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "string" | "number" | "boolean" | "void" | "object" | "unknown"
    )
}

fn is_builtin_type_token(type_name: &str) -> bool {
    matches!(
        type_name,
        "string"
            | "number"
            | "boolean"
            | "void"
            | "object"
            | "unknown"
            | "any"
            | "never"
            | "null"
            | "undefined"
            | "Array"
            | "ReadonlyArray"
            | "Record"
            | "Partial"
            | "Required"
            | "Pick"
            | "Omit"
            | "Exclude"
            | "Extract"
            | "NonNullable"
            | "Awaited"
            | "Promise"
            | "Date"
            | "File"
            | "Blob"
            | "FormData"
            | "Map"
            | "Set"
    )
}

fn extract_identifier_tokens(type_expr: &str) -> Vec<String> {
    let mut tokens = Vec::<String>::new();
    let mut current = String::new();

    for ch in type_expr.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            current.push(ch);
        } else if !current.is_empty() {
            if is_identifier_type(&current) && !tokens.iter().any(|item| item == &current) {
                tokens.push(current.clone());
            }
            current.clear();
        }
    }

    if !current.is_empty()
        && is_identifier_type(&current)
        && !tokens.iter().any(|item| item == &current)
    {
        tokens.push(current);
    }

    tokens
}

fn extract_referenced_types(type_name: &str) -> Vec<String> {
    let trimmed = type_name.trim();
    if trimmed.is_empty() {
        return vec![];
    }

    if is_identifier_type(trimmed) {
        if is_primitive_type(trimmed) || is_builtin_type_token(trimmed) {
            return vec![];
        }
        return vec![trimmed.to_string()];
    }

    let mut result = Vec::<String>::new();

    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        let inner = &trimmed[1..trimmed.len() - 1];
        for field in inner.split(';') {
            let field = field.trim();
            if field.is_empty() {
                continue;
            }
            if let Some((_, rhs)) = field.split_once(':') {
                for token in extract_identifier_tokens(rhs) {
                    if !is_builtin_type_token(&token)
                        && !is_primitive_type(&token)
                        && !result.iter().any(|item| item == &token)
                    {
                        result.push(token);
                    }
                }
            }
        }
        return result;
    }

    for token in extract_identifier_tokens(trimmed) {
        if !is_builtin_type_token(&token)
            && !is_primitive_type(&token)
            && !result.iter().any(|item| item == &token)
        {
            result.push(token);
        }
    }
    result
}

/// Renders a single type import line.
///
/// For package-style imports: `import type { TypeName } from "package-name";`
/// For relative imports: `import type { TypeName } from "path/TypeName";`
pub fn render_type_import_line(
    type_name: &str,
    base_import_path: &str,
    use_package: bool,
) -> String {
    let referenced_types = extract_referenced_types(type_name);
    referenced_types
        .iter()
        .map(|name| {
            if use_package {
                format!("import type {{ {name} }} from \"{base_import_path}\";\n")
            } else {
                format!("import type {{ {name} }} from \"{base_import_path}/{name}\";\n")
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

/// Renders a block of type imports, deduplicating types.
///
/// Filters out primitive types and invalid identifiers,
/// then generates import statements for each unique type.
pub fn render_type_import_block(
    type_names: &[&str],
    base_import_path: &str,
    use_package: bool,
) -> String {
    let mut unique = Vec::<String>::new();
    for type_name in type_names {
        for token in extract_referenced_types(type_name) {
            if !unique.iter().any(|item| item == &token) {
                unique.push(token);
            }
        }
    }
    unique
        .iter()
        .map(|type_name| render_type_import_line(type_name, base_import_path, use_package))
        .collect::<Vec<_>>()
        .join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_primitive_types() {
        assert_eq!(normalize_type_ref("string"), "string");
        assert_eq!(normalize_type_ref("number"), "number");
        assert_eq!(normalize_type_ref("boolean"), "boolean");
        assert_eq!(normalize_type_ref("void"), "void");
    }

    #[test]
    fn test_normalize_identifier_types() {
        assert_eq!(normalize_type_ref("MyType"), "MyType");
        assert_eq!(normalize_type_ref("_PrivateType"), "_PrivateType");
        assert_eq!(normalize_type_ref("Type123"), "Type123");
    }

    #[test]
    fn test_normalize_invalid_types() {
        assert_eq!(normalize_type_ref(""), "unknown");
        assert_eq!(normalize_type_ref("   "), "unknown");
    }

    #[test]
    fn test_normalize_complex_types() {
        assert_eq!(normalize_type_ref("MyType[]"), "MyType[]");
        assert_eq!(normalize_type_ref("string | number"), "string | number");
        assert_eq!(normalize_type_ref("{ id: string }"), "{ id: string }");
    }

    #[test]
    fn test_is_identifier_type() {
        assert!(is_identifier_type("MyType"));
        assert!(is_identifier_type("_private"));
        assert!(is_identifier_type("Type123"));
        assert!(!is_identifier_type("123Type"));
        assert!(is_identifier_type("a"));
    }

    #[test]
    fn test_is_primitive_type() {
        assert!(is_primitive_type("string"));
        assert!(is_primitive_type("number"));
        assert!(is_primitive_type("boolean"));
        assert!(is_primitive_type("void"));
        assert!(is_primitive_type("object"));
        assert!(is_primitive_type("unknown"));
        assert!(!is_primitive_type("String"));
        assert!(!is_primitive_type("MyType"));
    }

    #[test]
    fn test_render_type_import_line_package() {
        let result = render_type_import_line("MyType", "@my-org/models", true);
        assert_eq!(result, "import type { MyType } from \"@my-org/models\";\n");
    }

    #[test]
    fn test_render_type_import_line_relative() {
        let result = render_type_import_line("MyType", "../../../spec/types", false);
        assert_eq!(
            result,
            "import type { MyType } from \"../../../spec/types/MyType\";\n"
        );
    }

    #[test]
    fn test_render_type_import_line_primitive() {
        let result = render_type_import_line("string", "@my-org/models", true);
        assert_eq!(result, "");
    }

    #[test]
    fn test_render_type_import_block() {
        let result = render_type_import_block(
            &["MyType", "OtherType", "string", "MyType"], // duplicate MyType, primitive string
            "@my-org/models",
            true,
        );
        assert!(result.contains("import type { MyType }"));
        assert!(result.contains("import type { OtherType }"));
        assert!(!result.contains("string"));
    }

    #[test]
    fn test_render_type_import_line_inline_object_should_import_referenced_type() {
        let result = render_type_import_line(
            "{ StoreType: StoreType; body?: object }",
            "../../../spec/types",
            false,
        );
        assert!(
            result.contains("import type { StoreType } from \"../../../spec/types/StoreType\";")
        );
        assert!(!result.contains("body"));
    }

    #[test]
    fn test_render_type_import_block_should_extract_types_from_expression() {
        let result = render_type_import_block(
            &[
                "Record<string, any>",
                "StoreType | null",
                "{ value: GuidResultModel }",
            ],
            "@my-org/models",
            true,
        );
        assert!(result.contains("import type { StoreType }"));
        assert!(result.contains("import type { GuidResultModel }"));
        assert!(!result.contains("Record"));
    }
}
