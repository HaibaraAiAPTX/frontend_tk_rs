//! Import utilities for code generation.
//!
//! These utilities handle import path generation and client import configuration.

use std::path::{Component, Path, PathBuf};

use path_clean::PathClean;

use super::super::model::{ClientImportConfig, GeneratorInput, ModelImportConfig};

fn to_absolute_path(path: &Path) -> Option<PathBuf> {
    if path.is_absolute() {
        Some(path.clean())
    } else {
        std::env::current_dir()
            .ok()
            .map(|cwd| cwd.join(path).clean())
    }
}

fn relative_path(from_dir: &Path, to_path: &Path) -> Option<PathBuf> {
    let from_clean = from_dir.clean();
    let to_clean = to_path.clean();

    let from_components = from_clean.components().collect::<Vec<_>>();
    let to_components = to_clean.components().collect::<Vec<_>>();

    let common_len = from_components
        .iter()
        .zip(to_components.iter())
        .take_while(|(a, b)| a == b)
        .count();

    if common_len == 0 && (from_clean.is_absolute() || to_clean.is_absolute()) {
        return None;
    }

    let mut rel = PathBuf::new();

    for component in &from_components[common_len..] {
        if matches!(component, Component::Normal(_)) {
            rel.push("..");
        }
    }

    for component in &to_components[common_len..] {
        rel.push(component.as_os_str());
    }

    if rel.as_os_str().is_empty() {
        rel.push(".");
    }

    Some(rel)
}

fn to_import_path(path: &Path) -> String {
    let mut text = path.to_string_lossy().replace('\\', "/");

    if text.is_empty() {
        return "./".to_string();
    }

    if text == "." {
        return "./".to_string();
    }

    if !text.starts_with("./") && !text.starts_with("../") {
        text = format!("./{text}");
    }

    text
}

/// Calculates the model import base path for a generated file.
///
/// - `package` mode returns package path directly.
/// - `relative` mode resolves user path relative to execution dir and re-computes
///   a relative import path from each generated file.
fn calculate_relative_model_path(
    generated_file_path: &str,
    model_import: &Option<ModelImportConfig>,
    output_root: &Option<String>,
) -> String {
    let config = match model_import {
        None => return "../../../spec/types".to_string(),
        Some(c) => c,
    };

    if config.import_type == "package" {
        return config
            .package_path
            .clone()
            .unwrap_or_else(|| "@my-org/models".to_string());
    }

    let original_path = match &config.original_path {
        Some(p) if !p.trim().is_empty() => p.as_str(),
        _ => return "../../../spec/types".to_string(),
    };

    let output_root_text = match output_root {
        Some(v) if !v.trim().is_empty() => v,
        _ => return "../../../spec/types".to_string(),
    };

    let output_root_abs = match to_absolute_path(Path::new(output_root_text)) {
        Some(path) => path,
        None => return "../../../spec/types".to_string(),
    };

    let generated_file_abs = output_root_abs.join(generated_file_path).clean();
    let generated_dir_abs = generated_file_abs.parent().unwrap_or(&output_root_abs);

    let model_base_abs = if Path::new(original_path).is_absolute() {
        Path::new(original_path).clean()
    } else {
        match std::env::current_dir() {
            Ok(cwd) => cwd.join(original_path).clean(),
            Err(_) => return "../../../spec/types".to_string(),
        }
    };

    match relative_path(generated_dir_abs, &model_base_abs) {
        Some(path) => to_import_path(&path),
        None => original_path.replace('\\', "/"),
    }
}

/// Calculates the model import base path based on configuration.
///
/// # Returns
/// - For package imports: configured package path.
/// - For relative imports: calculated relative path based on generated file location.
/// - Fallback default: `../../../spec/types`.
pub fn get_model_import_base(
    model_import: &Option<ModelImportConfig>,
    generated_file_path: Option<&str>,
    output_root: &Option<String>,
) -> String {
    if let Some(file_path) = generated_file_path {
        return calculate_relative_model_path(file_path, model_import, output_root);
    }

    match model_import {
        None => "../../../spec/types".to_string(),
        Some(config) => match config.import_type.as_str() {
            "package" => config
                .package_path
                .clone()
                .unwrap_or_else(|| "@my-org/models".to_string()),
            "relative" => config
                .relative_path
                .clone()
                .unwrap_or_else(|| "../../../spec/types".to_string()),
            _ => "../../../spec/types".to_string(),
        },
    }
}

/// Unified entrypoint for resolving model import base from generator input.
pub fn resolve_model_import_base(input: &GeneratorInput, generated_file_path: &str) -> String {
    get_model_import_base(
        &input.model_import,
        Some(generated_file_path),
        &input.output_root,
    )
}

/// Resolve relative import path between two generated files.
///
/// Both paths are expected to be relative to the same output root.
/// Returns an import-ready path with `/` separators and no file extension.
pub fn resolve_file_import_path(from_file_path: &str, to_file_path: &str) -> String {
    let from_dir = Path::new(from_file_path).parent().unwrap_or(Path::new("."));
    let to_file = Path::new(to_file_path);

    let mut rel = relative_path(from_dir, to_file).unwrap_or_else(|| to_file.to_path_buf());
    rel.set_extension("");
    to_import_path(&rel)
}

/// Checks if package-style imports should be used.
///
/// Package-style imports don't include the type file suffix,
/// while relative imports do.
pub fn should_use_package_import(model_import: &Option<ModelImportConfig>) -> bool {
    match model_import {
        None => false,
        Some(config) => config.import_type == "package",
    }
}

/// Generates client import statements based on configuration.
///
/// # Modes
/// - `global`: Import from @aptx/api-client (default)
/// - `local`: Import from a local client path
/// - `package`: Import from a custom package
pub fn get_client_import_lines(client_import: &Option<ClientImportConfig>) -> String {
    match client_import {
        None => {
            // Default: global mode
            "import type { PerCallOptions } from \"@aptx/api-client\";\nimport { getApiClient } from \"@aptx/api-client\";".to_string()
        }
        Some(config) => {
            let import_name = config.import_name.as_deref().unwrap_or("getApiClient");
            match config.mode.as_str() {
                "global" => {
                    format!(
                        "import type {{ PerCallOptions }} from \"@aptx/api-client\";\nimport {{ {} }} from \"@aptx/api-client\";",
                        import_name
                    )
                }
                "local" => {
                    let client_path = config.client_path.as_deref().unwrap_or("../../api/client");
                    format!(
                        "import type {{ PerCallOptions }} from \"{}/types\";\nimport {{ {} }} from \"{}/client\";",
                        client_path, import_name, client_path
                    )
                }
                "package" => {
                    let package_name = config
                        .client_package
                        .as_deref()
                        .unwrap_or("@my-org/api-client");
                    format!(
                        "import type {{ PerCallOptions }} from \"{}/types\";\nimport {{ {} }} from \"{}/client\";",
                        package_name, import_name, package_name
                    )
                }
                _ => {
                    // Fallback to global
                    "import type { PerCallOptions } from \"@aptx/api-client\";\nimport { getApiClient } from \"@aptx/api-client\";".to_string()
                }
            }
        }
    }
}

/// Gets the client function call expression.
///
/// Returns the client function name with call syntax, e.g., "getApiClient()"
pub fn get_client_call(client_import: &Option<ClientImportConfig>) -> String {
    match client_import {
        None => "getApiClient()".to_string(),
        Some(config) => {
            let import_name = config.import_name.as_deref().unwrap_or("getApiClient");
            format!("{}()", import_name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_model_import_base_default() {
        let result = get_model_import_base(&None, None, &None);
        assert_eq!(result, "../../../spec/types");
    }

    #[test]
    fn test_get_model_import_base_package() {
        let config = ModelImportConfig {
            import_type: "package".to_string(),
            package_path: Some("@my-org/models".to_string()),
            relative_path: None,
            original_path: None,
        };
        let result = get_model_import_base(&Some(config), None, &None);
        assert_eq!(result, "@my-org/models");
    }

    #[test]
    fn test_get_model_import_base_relative() {
        let config = ModelImportConfig {
            import_type: "relative".to_string(),
            package_path: None,
            relative_path: Some("../../types".to_string()),
            original_path: None,
        };
        let result = get_model_import_base(&Some(config), None, &None);
        assert_eq!(result, "../../types");
    }

    #[test]
    fn test_calculate_relative_model_path_from_relative_model_dir() {
        let cwd = std::env::current_dir().expect("cwd");
        let output_root = cwd.join("target/test-output");
        let output_root_string = output_root.to_string_lossy().to_string();

        let config = ModelImportConfig {
            import_type: "relative".to_string(),
            package_path: None,
            relative_path: None,
            original_path: Some("domains".to_string()),
        };

        let result = get_model_import_base(
            &Some(config),
            Some("functions/user/getUser.ts"),
            &Some(output_root_string),
        );

        assert!(result.starts_with("../../../../"));
        assert!(result.ends_with("/domains"));
        assert!(!result.contains('\\'));
    }

    #[test]
    fn test_calculate_relative_model_path_from_absolute_model_dir() {
        let cwd = std::env::current_dir().expect("cwd");
        let output_root = cwd.join("target/test-output");
        let output_root_string = output_root.to_string_lossy().to_string();
        let absolute_model_path = cwd.join("domains").to_string_lossy().to_string();

        let config = ModelImportConfig {
            import_type: "relative".to_string(),
            package_path: None,
            relative_path: None,
            original_path: Some(absolute_model_path),
        };

        let result = get_model_import_base(
            &Some(config),
            Some("functions/user/getUser.ts"),
            &Some(output_root_string),
        );

        assert!(result.starts_with("../../../../"));
        assert!(result.ends_with("/domains"));
        assert!(!result.contains('\\'));
    }

    #[test]
    fn test_calculate_relative_model_path_keeps_relative_prefix() {
        let result = to_import_path(Path::new("domains"));
        assert_eq!(result, "./domains");
    }

    #[test]
    fn test_resolve_file_import_path_assignment_function_to_spec() {
        let result =
            resolve_file_import_path("functions/assignment/add.ts", "spec/assignment/add.ts");
        assert_eq!(result, "../../spec/assignment/add");
    }

    #[test]
    fn test_should_use_package_import() {
        assert!(!should_use_package_import(&None));

        let package_config = ModelImportConfig {
            import_type: "package".to_string(),
            package_path: None,
            relative_path: None,
            original_path: None,
        };
        assert!(should_use_package_import(&Some(package_config)));

        let relative_config = ModelImportConfig {
            import_type: "relative".to_string(),
            package_path: None,
            relative_path: None,
            original_path: None,
        };
        assert!(!should_use_package_import(&Some(relative_config)));
    }

    #[test]
    fn test_get_client_import_lines_default() {
        let result = get_client_import_lines(&None);
        assert!(result.contains("@aptx/api-client"));
        assert!(result.contains("getApiClient"));
    }

    #[test]
    fn test_get_client_call_default() {
        let result = get_client_call(&None);
        assert_eq!(result, "getApiClient()");
    }

    #[test]
    fn test_get_client_call_custom() {
        let config = ClientImportConfig {
            mode: "global".to_string(),
            import_name: Some("myClient".to_string()),
            client_path: None,
            client_package: None,
        };
        let result = get_client_call(&Some(config));
        assert_eq!(result, "myClient()");
    }
}
