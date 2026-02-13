//! Functions renderer for @aptx/api-client
//!
//! Generates TypeScript function calls that use @aptx/api-client for API execution.

use inflector::cases::pascalcase::to_pascal_case;

use crate::{
    get_client_call, get_client_import_lines, get_model_import_base, normalize_type_ref,
    render_type_import_block, render_type_import_line, should_use_package_import,
};

use swagger_gen::pipeline::{EndpointItem, GeneratorInput, PlannedFile, RenderOutput, Renderer};

/// Functions renderer for @aptx/api-client
///
/// Generates:
/// - Spec files in `spec/endpoints/{namespace}/{operation}.ts`
/// - Function files in `functions/api/{namespace}/{operation}.ts`
#[derive(Default)]
pub struct AptxFunctionsRenderer;

impl Renderer for AptxFunctionsRenderer {
    fn id(&self) -> &'static str {
        "aptx-functions"
    }

    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String> {
        let model_import_base = get_model_import_base(&input.model_import);
        let use_package = should_use_package_import(&input.model_import);
        let mut files = Vec::new();

        for endpoint in &input.endpoints {
            files.push(PlannedFile {
                path: get_spec_file_path(endpoint),
                content: render_spec_file(endpoint, &model_import_base, use_package),
            });
            files.push(PlannedFile {
                path: get_function_file_path(endpoint),
                content: render_function_file(
                    endpoint,
                    &model_import_base,
                    use_package,
                    &input.client_import,
                ),
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
    format!("spec/endpoints/{namespace}/{}.ts", endpoint.operation_name)
}

fn get_function_file_path(endpoint: &EndpointItem) -> String {
    let namespace = endpoint.namespace.join("/");
    format!("functions/api/{namespace}/{}.ts", endpoint.operation_name)
}

fn render_spec_file(endpoint: &EndpointItem, model_import_base: &str, use_package: bool) -> String {
    let builder = format!("build{}Spec", to_pascal_case(&endpoint.operation_name));
    let input_type = normalize_type_ref(&endpoint.input_type_name);
    let input_import = render_type_import_line(&input_type, model_import_base, use_package);
    let payload_field = endpoint
        .request_body_field
        .as_ref()
        .map(|field| format!("  body: (input as any)?.{},\n", field))
        .unwrap_or_default();
    let query_lines = if endpoint.query_fields.is_empty() {
        String::new()
    } else {
        let keys = endpoint
            .query_fields
            .iter()
            .map(|field| format!("{field}: (input as any)?.{field}"))
            .collect::<Vec<_>>()
            .join(", ");
        format!("  query: {{ {keys} }},\n")
    };

    let prefix = if input_import.is_empty() {
        String::new()
    } else {
        input_import
    };

    format!(
        "{prefix}export function {builder}(input: {input_type}) {{
  return {{
    method: \"{method}\",
    path: \"{path}\",
{query_lines}{payload_field}  }};
}}
",
        input_type = input_type,
        method = endpoint.method,
        path = endpoint.path
    )
}

fn render_function_file(
    endpoint: &EndpointItem,
    model_import_base: &str,
    use_package: bool,
    client_import: &Option<swagger_gen::pipeline::ClientImportConfig>,
) -> String {
    let builder = format!("build{}Spec", to_pascal_case(&endpoint.operation_name));
    let input_type = normalize_type_ref(&endpoint.input_type_name);
    let output_type = normalize_type_ref(&endpoint.output_type_name);
    let type_imports = render_type_import_block(
        &[input_type.as_str(), output_type.as_str()],
        model_import_base,
        use_package,
    );
    let client_import_lines = get_client_import_lines(client_import);
    let client_call = get_client_call(client_import);
    format!(
        "{client_import_lines}\nimport {{ {builder} }} from \"../../spec/endpoints/{namespace}/{operation_name}\";\n{type_imports}\n\nexport function {operation_name}(\n  input: {input_type},\n  options?: PerCallOptions\n): Promise<{output_type}> {{\n  return {client_call}.execute<{output_type}>({builder}(input), options);\n}}\n",
        namespace = endpoint.namespace.join("/"),
        operation_name = endpoint.operation_name,
        input_type = input_type,
        output_type = output_type,
        type_imports = type_imports,
        client_import_lines = client_import_lines,
        client_call = client_call,
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
        assert_eq!(get_spec_file_path(&endpoint), "spec/endpoints/users/getUser.ts");
    }

    #[test]
    fn test_get_function_file_path() {
        let endpoint = EndpointItem {
            namespace: vec!["users".to_string()],
            operation_name: "getUser".to_string(),
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
            "functions/api/users/getUser.ts"
        );
    }
}
