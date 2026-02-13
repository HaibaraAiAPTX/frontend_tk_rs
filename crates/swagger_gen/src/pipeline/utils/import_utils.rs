//! Import utilities for code generation.
//!
//! These utilities handle import path generation and client import configuration.

use super::super::model::{ClientImportConfig, ModelImportConfig};

/// Calculates the model import base path based on configuration.
///
/// # Arguments
/// * `model_import` - Optional model import configuration
///
/// # Returns
/// - For package imports: the configured package path (e.g., "@my-org/models")
/// - For relative imports: the configured relative path (e.g., "../../../spec/types")
/// - Default: "../../../spec/types"
pub fn get_model_import_base(model_import: &Option<ModelImportConfig>) -> String {
    match model_import {
        None => "../../../spec/types".to_string(), // Default backward-compatible path
        Some(config) => {
            match config.import_type.as_str() {
                "package" => {
                    // For package import, use the configured package path
                    config.package_path.clone().unwrap_or_else(|| "@my-org/models".to_string())
                }
                "relative" => {
                    // For relative import, use the configured relative path
                    config.relative_path.clone().unwrap_or_else(|| "../../../spec/types".to_string())
                }
                _ => "../../../spec/types".to_string(), // Fallback to default
            }
        }
    }
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
                    let package_name = config.client_package.as_deref().unwrap_or("@my-org/api-client");
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
        let result = get_model_import_base(&None);
        assert_eq!(result, "../../../spec/types");
    }

    #[test]
    fn test_get_model_import_base_package() {
        let config = ModelImportConfig {
            import_type: "package".to_string(),
            package_path: Some("@my-org/models".to_string()),
            relative_path: None,
        };
        let result = get_model_import_base(&Some(config));
        assert_eq!(result, "@my-org/models");
    }

    #[test]
    fn test_get_model_import_base_relative() {
        let config = ModelImportConfig {
            import_type: "relative".to_string(),
            package_path: None,
            relative_path: Some("../../types".to_string()),
        };
        let result = get_model_import_base(&Some(config));
        assert_eq!(result, "../../types");
    }

    #[test]
    fn test_should_use_package_import() {
        assert!(!should_use_package_import(&None));

        let package_config = ModelImportConfig {
            import_type: "package".to_string(),
            package_path: None,
            relative_path: None,
        };
        assert!(should_use_package_import(&Some(package_config)));

        let relative_config = ModelImportConfig {
            import_type: "relative".to_string(),
            package_path: None,
            relative_path: None,
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
