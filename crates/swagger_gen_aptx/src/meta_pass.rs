//! Meta field configuration for @aptx code generation.
//!
//! This module provides `AptxMetaPass` which sets meta fields for specific endpoints
//! based on naming patterns. For example, refresh token endpoints should skip the
//! auth refresh middleware to avoid infinite loops.

use crate::META_SKIP_AUTH_REFRESH;
use swagger_gen::pipeline::{EndpointItem, GeneratorInput, TransformPass};

/// Aptx-specific meta configuration pass.
///
/// ## Rules
///
/// 1. **Refresh Token endpoints** → `skipAuthRefresh: true`
///    - Matches endpoints whose operation_name or path ends with `RefreshToken`
///    - Example: `MainAPI/User/RefreshToken` → operation_name = `refreshToken`
///
/// ## Rationale
///
/// The auth middleware automatically refreshes expired tokens on 401 responses.
/// However, the refresh token endpoint itself should not trigger this behavior,
/// as it would cause an infinite loop if the refresh request fails.
pub struct AptxMetaPass;

impl TransformPass for AptxMetaPass {
    fn name(&self) -> &'static str {
        "aptx-meta"
    }

    fn apply(&self, input: &mut GeneratorInput) -> Result<(), String> {
        for endpoint in &mut input.endpoints {
            if is_refresh_token_endpoint(endpoint) {
                endpoint.meta.insert(
                    META_SKIP_AUTH_REFRESH.to_string(),
                    "true".to_string(),
                );
            }
        }
        Ok(())
    }
}

/// Determines if an endpoint is a refresh token endpoint.
///
/// Matches based on:
/// - `operation_name` ending with `RefreshToken` (case-insensitive)
/// - `path` ending with `/RefreshToken` (case-insensitive)
fn is_refresh_token_endpoint(endpoint: &EndpointItem) -> bool {
    let operation_lower = endpoint.operation_name.to_lowercase();
    let path_lower = endpoint.path.to_lowercase();

    // Check operation_name: e.g., "userRefreshToken", "refreshToken"
    if operation_lower.ends_with("refreshtoken") {
        return true;
    }

    // Check path: e.g., "/MainAPI/User/RefreshToken"
    if path_lower.ends_with("/refreshtoken") {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    fn create_test_endpoint(operation_name: &str, path: &str) -> EndpointItem {
        EndpointItem {
            namespace: vec!["test".to_string()],
            operation_name: operation_name.to_string(),
            export_name: format!("test{}", operation_name),
            builder_name: format!("buildTest{}Spec", operation_name),
            summary: None,
            method: "POST".to_string(),
            path: path.to_string(),
            input_type_name: "void".to_string(),
            output_type_name: "void".to_string(),
            request_body_field: None,
            query_fields: vec![],
            path_fields: vec![],
            has_request_options: false,
            deprecated: false,
            meta: IndexMap::new(),
        }
    }

    #[test]
    fn test_is_refresh_token_endpoint_by_operation_name() {
        // Should match
        assert!(is_refresh_token_endpoint(&create_test_endpoint(
            "userRefreshToken",
            "/user/refresh"
        )));
        assert!(is_refresh_token_endpoint(&create_test_endpoint(
            "refreshToken",
            "/auth/refresh"
        )));
        assert!(is_refresh_token_endpoint(&create_test_endpoint(
            "UserRefreshToken",
            "/auth/refresh"
        )));

        // Should not match
        assert!(!is_refresh_token_endpoint(&create_test_endpoint(
            "getUser",
            "/user/get"
        )));
        assert!(!is_refresh_token_endpoint(&create_test_endpoint(
            "createUser",
            "/user/create"
        )));
    }

    #[test]
    fn test_is_refresh_token_endpoint_by_path() {
        // Should match by path
        assert!(is_refresh_token_endpoint(&create_test_endpoint(
            "refresh",
            "/MainAPI/User/RefreshToken"
        )));
        assert!(is_refresh_token_endpoint(&create_test_endpoint(
            "refresh",
            "/api/auth/RefreshToken"
        )));

        // Should not match
        assert!(!is_refresh_token_endpoint(&create_test_endpoint(
            "getUser",
            "/user/get"
        )));
    }

    #[test]
    fn test_aptx_meta_pass() {
        let pass = AptxMetaPass;

        let mut input = GeneratorInput {
            project: swagger_gen::pipeline::ProjectContext {
                package_name: "test".to_string(),
                api_base_path: None,
                terminals: vec![],
                retry_ownership: None,
            },
            endpoints: vec![
                create_test_endpoint("userRefreshToken", "/user/refresh"),
                create_test_endpoint("getUser", "/user/get"),
                create_test_endpoint("refresh", "/MainAPI/User/RefreshToken"),
            ],
            model_import: None,
            client_import: None,
            output_root: None,
        };

        pass.apply(&mut input).unwrap();

        // userRefreshToken should have META_SKIP_AUTH_REFRESH = true
        assert!(input.endpoints[0].meta.get(META_SKIP_AUTH_REFRESH) == Some(&"true".to_string()));

        // getUser should not have META_SKIP_AUTH_REFRESH
        assert!(input.endpoints[1].meta.get(META_SKIP_AUTH_REFRESH).is_none());

        // path ending with /RefreshToken should have META_SKIP_AUTH_REFRESH = true
        assert!(input.endpoints[2].meta.get(META_SKIP_AUTH_REFRESH) == Some(&"true".to_string()));
    }
}
