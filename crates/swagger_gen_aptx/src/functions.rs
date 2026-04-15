//! Functions renderer for @aptx/api-client
//!
//! Generates TypeScript function calls that use @aptx/api-client for API execution.

use crate::META_SKIP_AUTH_REFRESH;
use crate::{
    ResolvedTsName, get_client_call, get_client_import_lines, normalize_type_ref,
    render_type_import_block, render_type_import_line, resolve_file_import_path,
    resolve_final_ts_names, resolve_model_import_base, should_use_package_import,
};

use swagger_gen::pipeline::{EndpointItem, GeneratorInput, PlannedFile, RenderOutput, Renderer};

/// Functions renderer for @aptx/api-client
///
/// Generates:
/// - Spec files in `spec/{namespace}/{operation}.ts`
/// - Function files in `functions/{namespace}/{operation}.ts`
#[derive(Default)]
pub struct AptxFunctionsRenderer;

impl Renderer for AptxFunctionsRenderer {
    fn id(&self) -> &'static str {
        "aptx-functions"
    }

    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String> {
        let use_package = should_use_package_import(&input.model_import);
        let mut files = Vec::new();
        let resolved_names = resolve_final_ts_names(&input.endpoints);

        for (endpoint, resolved_name) in input.endpoints.iter().zip(resolved_names.iter()) {
            let spec_path = get_spec_file_path(endpoint, resolved_name);
            let function_path = get_function_file_path(endpoint, resolved_name);

            // Calculate correct relative path for each file
            let spec_model_import_base = resolve_model_import_base(input, &spec_path);
            let function_model_import_base = resolve_model_import_base(input, &function_path);

            files.push(PlannedFile {
                path: spec_path,
                content: render_spec_file(
                    endpoint,
                    resolved_name,
                    &spec_model_import_base,
                    use_package,
                ),
            });
            let function_content = render_function_file(
                endpoint,
                resolved_name,
                &function_path,
                &function_model_import_base,
                use_package,
                &input.client_import,
            );
            files.push(PlannedFile {
                path: function_path,
                content: function_content,
            });
        }

        Ok(RenderOutput {
            files,
            warnings: vec![],
        })
    }
}

fn get_spec_file_path(endpoint: &EndpointItem, resolved_name: &ResolvedTsName) -> String {
    let namespace = endpoint.namespace.join("/");
    format!("spec/{namespace}/{}.ts", resolved_name.file_stem)
}

fn get_function_file_path(endpoint: &EndpointItem, resolved_name: &ResolvedTsName) -> String {
    let namespace = endpoint.namespace.join("/");
    format!("functions/{namespace}/{}.ts", resolved_name.file_stem)
}

fn render_spec_file(
    endpoint: &EndpointItem,
    resolved_name: &ResolvedTsName,
    model_import_base: &str,
    use_package: bool,
) -> String {
    let builder = resolved_name.builder_name.clone();
    let input_type = normalize_type_ref(&endpoint.input_type_name);
    let input_import = render_type_import_line(&input_type, model_import_base, use_package);
    let is_void_input = input_type == "void";
    let signature = if is_void_input {
        String::new()
    } else {
        format!("input: {input_type}")
    };
    let payload_field = endpoint
        .request_body_field
        .as_ref()
        .map(|field| {
            if is_void_input {
                String::new()
            } else if endpoint.query_fields.is_empty() && endpoint.path_fields.is_empty() {
                "    body: input,\n".to_string()
            } else {
                format!("    body: input.{field},\n")
            }
        })
        .unwrap_or_default();
    let query_lines = if endpoint.query_fields.is_empty() || is_void_input {
        String::new()
    } else {
        let keys = endpoint
            .query_fields
            .iter()
            .map(|field| format!("{field}: input.{field}"))
            .collect::<Vec<_>>()
            .join(", ");
        format!("    query: {{ {keys} }},\n")
    };

    // Check if endpoint has skip_auth_refresh meta
    let has_skip_auth_refresh =
        endpoint.meta.get(META_SKIP_AUTH_REFRESH) == Some(&"true".to_string());

    // Collect all non-internal meta fields (keys not starting with "__")
    let meta_fields: Vec<_> = endpoint
        .meta
        .iter()
        .filter(|(k, _)| !k.starts_with("__"))
        .collect();

    // Build meta field for RequestSpec
    let meta_field = if !meta_fields.is_empty() {
        let fields: Vec<String> = meta_fields
            .iter()
            .map(|(k, v)| {
                // For SKIP_AUTH_REFRESH_META_KEY, use computed property syntax
                if k.as_str() == META_SKIP_AUTH_REFRESH {
                    format!("[{k}]: {v}")
                } else {
                    format!("{k}: {v}")
                }
            })
            .collect();
        format!("    meta: {{ {} }},\n", fields.join(", "))
    } else {
        String::new()
    };

    // Build imports
    let mut imports = Vec::new();

    // Add input type import if needed
    if !input_import.is_empty() {
        imports.push(input_import);
    }

    // Add RequestSpec import (and SKIP_AUTH_REFRESH_META_KEY if needed)
    if has_skip_auth_refresh {
        imports.push(
            "import { SKIP_AUTH_REFRESH_META_KEY } from \"@aptx/api-plugin-auth\";".to_string(),
        );
        imports.push("import type { RequestSpec } from \"@aptx/api-client\";".to_string());
    } else {
        imports.push("import type { RequestSpec } from \"@aptx/api-client\";".to_string());
    }

    let prefix = if imports.is_empty() {
        String::new()
    } else {
        format!("{}\n\n", imports.join("\n"))
    };

    format!(
        "{prefix}export function {builder}({signature}): RequestSpec {{\n  return {{\n    method: \"{method}\",\n    path: \"{path}\",\n{query_lines}{payload_field}{meta_field}  }};\n}}\n",
        signature = signature,
        method = endpoint.method,
        path = endpoint.path
    )
}

fn render_function_file(
    endpoint: &EndpointItem,
    resolved_name: &ResolvedTsName,
    current_file_path: &str,
    model_import_base: &str,
    use_package: bool,
    client_import: &Option<swagger_gen::pipeline::ClientImportConfig>,
) -> String {
    let builder = resolved_name.builder_name.clone();
    let input_type = normalize_type_ref(&endpoint.input_type_name);
    let output_type = normalize_type_ref(&endpoint.output_type_name);
    let is_void_input = input_type == "void";
    let input_signature = if is_void_input {
        String::new()
    } else {
        format!("  input: {input_type},\n")
    };
    let builder_call = if is_void_input {
        format!("{builder}()")
    } else {
        format!("{builder}(input)")
    };
    let type_imports = render_type_import_block(
        &[input_type.as_str(), output_type.as_str()],
        model_import_base,
        use_package,
    );
    let spec_file_path = get_spec_file_path(endpoint, resolved_name);
    let spec_import_path = resolve_file_import_path(current_file_path, &spec_file_path);
    let client_import_lines = get_client_import_lines(client_import);
    let client_call = get_client_call(client_import);
    let type_import_block = if type_imports.is_empty() {
        "\n".to_string()
    } else {
        format!("{type_imports}\n")
    };
    format!(
        "{client_import_lines}\nimport {{ {builder} }} from \"{spec_import_path}\";\n{type_import_block}export function {operation_name}(\n{input_signature}  options?: PerCallOptions\n): Promise<{output_type}> {{\n  return {client_call}.execute<{output_type}>({builder_call}, options);\n}}\n",
        operation_name = resolved_name.export_name,
        output_type = output_type,
        type_import_block = type_import_block,
        client_import_lines = client_import_lines,
        client_call = client_call,
        input_signature = input_signature,
        builder_call = builder_call,
        spec_import_path = spec_import_path,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    use swagger_gen::pipeline::ProjectContext;

    fn make_generator_input(endpoints: Vec<EndpointItem>) -> GeneratorInput {
        GeneratorInput {
            project: ProjectContext {
                package_name: "test".to_string(),
                api_base_path: None,
                terminals: vec![],
                retry_ownership: None,
            },
            endpoints,
            model_import: None,
            client_import: None,
            output_root: None,
        }
    }

    #[test]
    fn test_renderer_id() {
        let renderer = AptxFunctionsRenderer;
        assert_eq!(renderer.id(), "aptx-functions");
    }

    #[test]
    fn test_get_spec_file_path() {
        let endpoint = EndpointItem {
            namespace: vec!["users".to_string()],
            operation_name: "getUser".to_string(),
            export_name: "usersGetUser".to_string(),
            builder_name: "buildUsersGetUserSpec".to_string(),
            summary: None,
            method: "GET".to_string(),
            path: "/users/{id}".to_string(),
            input_type_name: "GetUserInput".to_string(),
            output_type_name: "User".to_string(),
            request_body_field: None,
            query_params: vec![],
            query_fields: vec![],
            path_params: vec![],
            path_fields: vec!["id".to_string()],
            has_request_options: false,
            deprecated: false,
            meta: IndexMap::new(),
        };
        let resolved_name = ResolvedTsName {
            file_stem: "getUser".to_string(),
            export_name: "getUser".to_string(),
            builder_name: "buildGetUserSpec".to_string(),
        };
        assert_eq!(
            get_spec_file_path(&endpoint, &resolved_name),
            "spec/users/getUser.ts"
        );
    }

    #[test]
    fn test_get_function_file_path() {
        let endpoint = EndpointItem {
            namespace: vec!["users".to_string()],
            operation_name: "getUser".to_string(),
            export_name: "usersGetUser".to_string(),
            builder_name: "buildUsersGetUserSpec".to_string(),
            summary: None,
            method: "GET".to_string(),
            path: "/users/{id}".to_string(),
            input_type_name: "GetUserInput".to_string(),
            output_type_name: "User".to_string(),
            request_body_field: None,
            query_params: vec![],
            query_fields: vec![],
            path_params: vec![],
            path_fields: vec!["id".to_string()],
            has_request_options: false,
            deprecated: false,
            meta: IndexMap::new(),
        };
        let resolved_name = ResolvedTsName {
            file_stem: "getUser".to_string(),
            export_name: "getUser".to_string(),
            builder_name: "buildGetUserSpec".to_string(),
        };
        assert_eq!(
            get_function_file_path(&endpoint, &resolved_name),
            "functions/users/getUser.ts"
        );
    }

    #[test]
    fn test_render_function_file_imports_spec_with_dynamic_relative_path() {
        let endpoint = EndpointItem {
            namespace: vec!["assignment".to_string()],
            operation_name: "add".to_string(),
            export_name: "assignmentAdd".to_string(),
            builder_name: "buildAssignmentAddSpec".to_string(),
            summary: None,
            method: "POST".to_string(),
            path: "/assignment/add".to_string(),
            input_type_name: "AddInput".to_string(),
            output_type_name: "AddOutput".to_string(),
            request_body_field: None,
            query_params: vec![],
            query_fields: vec![],
            path_params: vec![],
            path_fields: vec![],
            has_request_options: false,
            deprecated: false,
            meta: IndexMap::new(),
        };
        let content = render_function_file(
            &endpoint,
            &ResolvedTsName {
                file_stem: "add".to_string(),
                export_name: "add".to_string(),
                builder_name: "buildAddSpec".to_string(),
            },
            "functions/assignment/add.ts",
            "../../../spec/types",
            false,
            &None,
        );

        assert!(content.contains("from \"../../spec/assignment/add\""));
    }

    #[test]
    fn test_render_spec_file_should_import_nested_model_types() {
        let endpoint = EndpointItem {
            namespace: vec!["stored-file".to_string()],
            operation_name: "uploadImage".to_string(),
            export_name: "storedFileUploadImage".to_string(),
            builder_name: "buildStoredFileUploadImageSpec".to_string(),
            summary: None,
            method: "POST".to_string(),
            path: "/stored-file/upload".to_string(),
            input_type_name: "{ StoreType: StoreType; body?: object }".to_string(),
            output_type_name: "GuidResultModel".to_string(),
            request_body_field: Some("body".to_string()),
            query_params: vec![],
            query_fields: vec!["StoreType".to_string()],
            path_params: vec![],
            path_fields: vec![],
            has_request_options: false,
            deprecated: false,
            meta: IndexMap::new(),
        };

        let content = render_spec_file(
            &endpoint,
            &ResolvedTsName {
                file_stem: "uploadImage".to_string(),
                export_name: "uploadImage".to_string(),
                builder_name: "buildUploadImageSpec".to_string(),
            },
            "../../../domains",
            false,
        );
        assert!(content.contains("import type { StoreType } from \"../../../domains/StoreType\";"));
    }

    #[test]
    fn test_render_spec_file_with_skip_auth_refresh() {
        let mut meta = IndexMap::new();
        meta.insert(META_SKIP_AUTH_REFRESH.to_string(), "true".to_string());

        let endpoint = EndpointItem {
            namespace: vec!["user".to_string()],
            operation_name: "refreshToken".to_string(),
            export_name: "userRefreshToken".to_string(),
            builder_name: "buildUserRefreshTokenSpec".to_string(),
            summary: None,
            method: "POST".to_string(),
            path: "/MainAPI/User/RefreshToken".to_string(),
            input_type_name: "RefreshTokenInput".to_string(),
            output_type_name: "RefreshTokenOutput".to_string(),
            request_body_field: Some("body".to_string()),
            query_params: vec![],
            query_fields: vec![],
            path_params: vec![],
            path_fields: vec![],
            has_request_options: false,
            deprecated: false,
            meta,
        };

        let content = render_spec_file(
            &endpoint,
            &ResolvedTsName {
                file_stem: "refreshToken".to_string(),
                export_name: "refreshToken".to_string(),
                builder_name: "buildRefreshTokenSpec".to_string(),
            },
            "../../../domains",
            false,
        );

        // Should import SKIP_AUTH_REFRESH_META_KEY
        assert!(
            content
                .contains("import { SKIP_AUTH_REFRESH_META_KEY } from \"@aptx/api-plugin-auth\";")
        );

        // Should include meta field with computed key
        assert!(content.contains("meta: { [SKIP_AUTH_REFRESH_META_KEY]: true }"));
    }

    #[test]
    fn test_render_spec_file_without_skip_auth_refresh() {
        let endpoint = EndpointItem {
            namespace: vec!["user".to_string()],
            operation_name: "getUser".to_string(),
            export_name: "userGetUser".to_string(),
            builder_name: "buildUserGetUserSpec".to_string(),
            summary: None,
            method: "GET".to_string(),
            path: "/user/{id}".to_string(),
            input_type_name: "void".to_string(),
            output_type_name: "User".to_string(),
            request_body_field: None,
            query_params: vec![],
            query_fields: vec![],
            path_params: vec![],
            path_fields: vec!["id".to_string()],
            has_request_options: false,
            deprecated: false,
            meta: IndexMap::new(),
        };

        let content = render_spec_file(
            &endpoint,
            &ResolvedTsName {
                file_stem: "getUser".to_string(),
                export_name: "getUser".to_string(),
                builder_name: "buildGetUserSpec".to_string(),
            },
            "../../../domains",
            false,
        );

        // Should NOT import SKIP_AUTH_REFRESH_META_KEY
        assert!(!content.contains("SKIP_AUTH_REFRESH_META_KEY"));

        // Should NOT include meta field
        assert!(!content.contains("meta:"));
    }

    #[test]
    fn test_renderer_uses_short_names_inside_namespace_directory() {
        let input = make_generator_input(vec![
            EndpointItem {
                namespace: vec!["action_authority".to_string()],
                operation_name: "postAuthorityAPIActionAuthorityAdd".to_string(),
                export_name: "actionAuthorityAdd".to_string(),
                builder_name: "buildActionAuthorityAddSpec".to_string(),
                summary: None,
                method: "POST".to_string(),
                path: "/AuthorityAPI/ActionAuthority/Add".to_string(),
                input_type_name: "AddActionAuthorityRequestModel".to_string(),
                output_type_name: "GuidResultModel".to_string(),
                request_body_field: None,
                query_params: vec![],
                query_fields: vec![],
                path_params: vec![],
                path_fields: vec![],
                has_request_options: false,
                deprecated: false,
                meta: IndexMap::new(),
            },
            EndpointItem {
                namespace: vec!["role".to_string()],
                operation_name: "postAuthorityAPIRoleAdd".to_string(),
                export_name: "roleAdd".to_string(),
                builder_name: "buildRoleAddSpec".to_string(),
                summary: None,
                method: "POST".to_string(),
                path: "/AuthorityAPI/Role/Add".to_string(),
                input_type_name: "AddRoleRequestModel".to_string(),
                output_type_name: "GuidResultModel".to_string(),
                request_body_field: None,
                query_params: vec![],
                query_fields: vec![],
                path_params: vec![],
                path_fields: vec![],
                has_request_options: false,
                deprecated: false,
                meta: IndexMap::new(),
            },
        ]);

        let output = AptxFunctionsRenderer.render(&input).unwrap();
        assert!(
            output
                .files
                .iter()
                .any(|f| f.path == "functions/action_authority/add.ts")
        );
        assert!(
            output
                .files
                .iter()
                .any(|f| f.path == "spec/action_authority/add.ts")
        );
        assert!(
            output
                .files
                .iter()
                .any(|f| f.path == "functions/role/add.ts")
        );

        let action_function = output
            .files
            .iter()
            .find(|f| f.path == "functions/action_authority/add.ts")
            .expect("action authority function");
        assert!(
            action_function
                .content
                .contains("import { buildActionAuthorityAddSpec }")
        );
        assert!(
            action_function
                .content
                .contains("export function actionAuthorityAdd(")
        );
    }

    #[test]
    fn test_renderer_prefixes_single_namespace_function_exports() {
        let input = make_generator_input(vec![EndpointItem {
            namespace: vec!["announcement".to_string()],
            operation_name: "postAuthorityAPIAnnouncementAdd".to_string(),
            export_name: "announcementAdd".to_string(),
            builder_name: "buildAnnouncementAddSpec".to_string(),
            summary: None,
            method: "POST".to_string(),
            path: "/AuthorityAPI/Announcement/Add".to_string(),
            input_type_name: "AddAnnouncementRequestModel".to_string(),
            output_type_name: "GuidResultModel".to_string(),
            request_body_field: None,
            query_params: vec![],
            query_fields: vec![],
            path_params: vec![],
            path_fields: vec![],
            has_request_options: false,
            deprecated: false,
            meta: IndexMap::new(),
        }]);

        let output = AptxFunctionsRenderer.render(&input).unwrap();
        let function = output
            .files
            .iter()
            .find(|f| f.path == "functions/announcement/add.ts")
            .expect("announcement function");

        assert!(
            function
                .content
                .contains("import { buildAnnouncementAddSpec }")
        );
        assert!(
            function
                .content
                .contains("export function announcementAdd(")
        );
        assert!(!function.content.contains("export function add("));
    }

    #[test]
    fn test_renderer_keeps_action_body_after_namespace_prefix() {
        let input = make_generator_input(vec![
            EndpointItem {
                namespace: vec!["user".to_string()],
                operation_name: "getAuthorityAPIUserGetLoginUserInfo".to_string(),
                export_name: "userGetLoginUserInfo".to_string(),
                builder_name: "buildUserGetLoginUserInfoSpec".to_string(),
                summary: None,
                method: "GET".to_string(),
                path: "/AuthorityAPI/User/GetLoginUserInfo".to_string(),
                input_type_name: "void".to_string(),
                output_type_name: "LoginUserInfo".to_string(),
                request_body_field: None,
                query_params: vec![],
                query_fields: vec![],
                path_params: vec![],
                path_fields: vec![],
                has_request_options: false,
                deprecated: false,
                meta: IndexMap::new(),
            },
            EndpointItem {
                namespace: vec!["user".to_string()],
                operation_name: "getAuthorityAPIUserGetLoginUserPermissions".to_string(),
                export_name: "userGetLoginUserPermissions".to_string(),
                builder_name: "buildUserGetLoginUserPermissionsSpec".to_string(),
                summary: None,
                method: "GET".to_string(),
                path: "/AuthorityAPI/User/GetLoginUserPermissions".to_string(),
                input_type_name: "void".to_string(),
                output_type_name: "LoginUserPermissions".to_string(),
                request_body_field: None,
                query_params: vec![],
                query_fields: vec![],
                path_params: vec![],
                path_fields: vec![],
                has_request_options: false,
                deprecated: false,
                meta: IndexMap::new(),
            },
        ]);

        let output = AptxFunctionsRenderer.render(&input).unwrap();
        assert!(
            output
                .files
                .iter()
                .any(|f| f.path == "functions/user/getLoginUserInfo.ts")
        );
        assert!(
            output
                .files
                .iter()
                .any(|f| f.path == "spec/user/getLoginUserInfo.ts")
        );
        assert!(
            output
                .files
                .iter()
                .any(|f| f.path == "functions/user/getLoginUserPermissions.ts")
        );
        assert!(
            !output
                .files
                .iter()
                .any(|f| f.path == "functions/user/info.ts")
        );
        assert!(
            !output
                .files
                .iter()
                .any(|f| f.path == "functions/user/permissions.ts")
        );
    }

    #[test]
    fn test_renderer_name_collision_falls_back_to_long_name_within_namespace() {
        let input = make_generator_input(vec![
            EndpointItem {
                namespace: vec!["user".to_string()],
                operation_name: "postAuthorityAPIUserAdd".to_string(),
                export_name: "userAdd".to_string(),
                builder_name: "buildUserAddSpec".to_string(),
                summary: None,
                method: "POST".to_string(),
                path: "/AuthorityAPI/User/Add".to_string(),
                input_type_name: "AddUserRequest".to_string(),
                output_type_name: "User".to_string(),
                request_body_field: None,
                query_params: vec![],
                query_fields: vec![],
                path_params: vec![],
                path_fields: vec![],
                has_request_options: false,
                deprecated: false,
                meta: IndexMap::new(),
            },
            EndpointItem {
                namespace: vec!["user".to_string()],
                operation_name: "getAuthorityAPIUserAdd".to_string(),
                export_name: "userAddGet".to_string(),
                builder_name: "buildUserAddGetSpec".to_string(),
                summary: None,
                method: "GET".to_string(),
                path: "/AuthorityAPI/User/Add".to_string(),
                input_type_name: "void".to_string(),
                output_type_name: "User".to_string(),
                request_body_field: None,
                query_params: vec![],
                query_fields: vec![],
                path_params: vec![],
                path_fields: vec![],
                has_request_options: false,
                deprecated: false,
                meta: IndexMap::new(),
            },
        ]);

        let output = AptxFunctionsRenderer.render(&input).unwrap();
        assert!(
            output
                .files
                .iter()
                .any(|f| f.path == "functions/user/userAdd.ts")
        );
        assert!(
            output
                .files
                .iter()
                .any(|f| f.path == "functions/user/userAddGet.ts")
        );
    }
}
