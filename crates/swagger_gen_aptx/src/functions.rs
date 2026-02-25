//! Functions renderer for @aptx/api-client
//!
//! Generates TypeScript function calls that use @aptx/api-client for API execution.

use crate::{
    get_client_call, get_client_import_lines, normalize_type_ref, render_type_import_block,
    render_type_import_line, resolve_file_import_path, resolve_model_import_base,
    should_use_package_import,
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

        for endpoint in &input.endpoints {
            let spec_path = get_spec_file_path(endpoint);
            let function_path = get_function_file_path(endpoint);

            // Calculate correct relative path for each file
            let spec_model_import_base = resolve_model_import_base(input, &spec_path);
            let function_model_import_base = resolve_model_import_base(input, &function_path);

            files.push(PlannedFile {
                path: spec_path,
                content: render_spec_file(endpoint, &spec_model_import_base, use_package),
            });
            let function_content = render_function_file(
                endpoint,
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

fn get_spec_file_path(endpoint: &EndpointItem) -> String {
    let namespace = endpoint.namespace.join("/");
    format!("spec/{namespace}/{}.ts", endpoint.operation_name)
}

fn get_function_file_path(endpoint: &EndpointItem) -> String {
    let namespace = endpoint.namespace.join("/");
    format!("functions/{namespace}/{}.ts", endpoint.operation_name)
}

fn render_spec_file(endpoint: &EndpointItem, model_import_base: &str, use_package: bool) -> String {
    let builder = endpoint.builder_name.clone();
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

    let prefix = if input_import.is_empty() {
        "import type { RequestSpec } from \"@aptx/api-client\";\n\n".to_string()
    } else {
        format!("{input_import}\nimport type {{ RequestSpec }} from \"@aptx/api-client\";\n\n")
    };

    format!(
        "{prefix}export function {builder}({signature}): RequestSpec {{\n  return {{\n    method: \"{method}\",\n    path: \"{path}\",\n{query_lines}{payload_field}  }};\n}}\n",
        signature = signature,
        method = endpoint.method,
        path = endpoint.path
    )
}

fn render_function_file(
    endpoint: &EndpointItem,
    current_file_path: &str,
    model_import_base: &str,
    use_package: bool,
    client_import: &Option<swagger_gen::pipeline::ClientImportConfig>,
) -> String {
    let builder = endpoint.builder_name.clone();
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
    let spec_file_path = get_spec_file_path(endpoint);
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
        operation_name = endpoint.export_name,
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
            query_fields: vec![],
            path_fields: vec!["id".to_string()],
            has_request_options: false,
            supports_query: false,
            supports_mutation: false,
            deprecated: false,
        };
        assert_eq!(get_spec_file_path(&endpoint), "spec/users/getUser.ts");
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
            query_fields: vec![],
            path_fields: vec!["id".to_string()],
            has_request_options: false,
            supports_query: false,
            supports_mutation: false,
            deprecated: false,
        };
        assert_eq!(
            get_function_file_path(&endpoint),
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
            query_fields: vec![],
            path_fields: vec![],
            has_request_options: false,
            supports_query: false,
            supports_mutation: false,
            deprecated: false,
        };
        let content = render_function_file(
            &endpoint,
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
            query_fields: vec!["StoreType".to_string()],
            path_fields: vec![],
            has_request_options: false,
            supports_query: false,
            supports_mutation: true,
            deprecated: false,
        };

        let content = render_spec_file(&endpoint, "../../../domains", false);
        assert!(content.contains("import type { StoreType } from \"../../../domains/StoreType\";"));
    }
}
