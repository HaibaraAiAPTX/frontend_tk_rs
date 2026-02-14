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

/// Renders a single type import line.
///
/// For package-style imports: `import type { TypeName } from "package-name";`
/// For relative imports: `import type { TypeName } from "path/TypeName";`
pub fn render_type_import_line(
    type_name: &str,
    base_import_path: &str,
    use_package: bool,
) -> String {
    if is_identifier_type(type_name) && !is_primitive_type(type_name) {
        if use_package {
            // Package-style import: import type { TypeName } from "package-name"
            format!("import type {{ {type_name} }} from \"{base_import_path}\";\n")
        } else {
            // Relative-style import: import type { TypeName } from "path/TypeName"
            format!("import type {{ {type_name} }} from \"{base_import_path}/{type_name}\";\n")
        }
    } else {
        String::new()
    }
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
        if is_identifier_type(type_name)
            && !is_primitive_type(type_name)
            && !unique.iter().any(|item| item == type_name)
        {
            unique.push(type_name.to_string());
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
}
