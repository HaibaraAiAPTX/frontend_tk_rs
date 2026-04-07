//! Python functions renderer for swagger_gen.
//!
//! Generates spec files and function files that use aptx_api_core.

use swagger_gen::pipeline::{EndpointItem, GeneratorInput, PlannedFile, RenderOutput, Renderer};

/// Renderer that generates Python spec + function files.
#[derive(Default)]
pub struct PythonFunctionsRenderer;

impl Renderer for PythonFunctionsRenderer {
    fn id(&self) -> &'static str {
        "python-functions"
    }

    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String> {
        let common_prefix = find_common_service_prefix(&input.endpoints);
        let mut files = Vec::new();

        for endpoint in &input.endpoints {
            let py_name = compute_py_name(endpoint, &common_prefix);
            files.push(PlannedFile {
                path: get_spec_file_path(endpoint, &py_name),
                content: render_spec_file(endpoint, &py_name),
            });
            files.push(PlannedFile {
                path: get_function_file_path(endpoint, &py_name),
                content: render_function_file(endpoint, &py_name),
            });
        }

        Ok(RenderOutput {
            files,
            warnings: vec![],
        })
    }
}

fn get_spec_file_path(endpoint: &EndpointItem, py_name: &str) -> String {
    let namespace: Vec<String> = endpoint.namespace.iter().map(|s| escape_keyword(s)).collect();
    let namespace = namespace.join("/");
    format!("spec/{namespace}/{py_name}_spec.py")
}

fn get_function_file_path(endpoint: &EndpointItem, py_name: &str) -> String {
    let namespace: Vec<String> = endpoint.namespace.iter().map(|s| escape_keyword(s)).collect();
    let namespace = namespace.join("/");
    format!("functions/{namespace}/{py_name}.py")
}

const PYTHON_KEYWORDS: &[&str] = &[
    "False", "None", "True", "and", "as", "assert", "async", "await",
    "break", "class", "continue", "def", "del", "elif", "else", "except",
    "finally", "for", "from", "global", "if", "import", "in", "is",
    "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try",
    "while", "with", "yield",
];

fn escape_keyword(name: &str) -> String {
    let escaped = name.replace('-', "_");
    if PYTHON_KEYWORDS.contains(&escaped.as_str()) {
        format!("{escaped}_")
    } else {
        escaped
    }
}

/// Extract the part after the HTTP method prefix (leading lowercase run).
fn extract_service_part(name: &str) -> &str {
    let idx = name.find(|c: char| c.is_uppercase()).unwrap_or(0);
    &name[idx..]
}

/// Find the longest common prefix across all endpoints' service parts.
/// Returns empty string if fewer than 2 endpoints or prefix would leave any endpoint empty.
fn find_common_service_prefix(endpoints: &[EndpointItem]) -> String {
    if endpoints.len() <= 1 {
        return String::new();
    }

    let parts: Vec<&str> = endpoints
        .iter()
        .map(|e| extract_service_part(&e.operation_name))
        .collect();

    let mut prefix = parts[0].to_string();
    for part in &parts[1..] {
        while !part.starts_with(&prefix) {
            prefix.pop();
            if prefix.is_empty() {
                return String::new();
            }
        }
    }

    // Don't strip if any endpoint would be left with nothing
    if parts.iter().any(|p| p.strip_prefix(&prefix).unwrap_or("").is_empty()) {
        return String::new();
    }

    prefix
}

/// Compute the Python operation name:
/// 1. Skip the HTTP method prefix (get/post/put/delete)
/// 2. Strip the common service prefix (e.g. "EducationalAPI")
/// 3. Strip the namespace prefix (e.g. "Class" for namespace ["class"])
/// 4. Convert remaining action to snake_case
fn compute_py_name(endpoint: &EndpointItem, common_prefix: &str) -> String {
    let method_end = endpoint.operation_name.find(|c: char| c.is_uppercase()).unwrap_or(0);
    let service_part = &endpoint.operation_name[method_end..]; // skip HTTP method

    // Strip common service prefix
    let after_service = service_part.strip_prefix(common_prefix).unwrap_or(service_part);

    // Strip namespace prefix
    let ns_camel = namespace_to_camel(&endpoint.namespace);
    let action = after_service.strip_prefix(&ns_camel).unwrap_or(after_service);

    // If action is empty after stripping, fall back to full name after service prefix
    if action.is_empty() {
        return to_snake_case(after_service);
    }

    to_snake_case(action)
}

/// Convert namespace segments to CamelCase for prefix matching.
/// ["class-schedule"] → "ClassSchedule", ["class"] → "Class"
fn namespace_to_camel(namespace: &[String]) -> String {
    namespace
        .join("-")
        .split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect()
}

fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = name.chars().collect();
    for (i, ch) in chars.iter().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                let prev = chars[i - 1];
                let next_is_lower = chars.get(i + 1).map_or(false, |c| c.is_lowercase());
                if prev.is_lowercase() || next_is_lower {
                    result.push('_');
                }
            }
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(*ch);
        }
    }
    result
}

fn to_snake_module(name: &str) -> String {
    to_snake_case(name)
}

fn is_void_input(input_type_name: &str) -> bool {
    input_type_name == "void"
}

fn is_inline_input(input_type_name: &str) -> bool {
    input_type_name.contains('{') || input_type_name.contains(';')
}

fn parse_inline_fields(input_type_name: &str) -> Vec<(String, String)> {
    let trimmed = input_type_name.trim();
    if !trimmed.starts_with('{') {
        return vec![];
    }
    let inner = trimmed.trim_start_matches('{').trim_end_matches('}').trim();
    if inner.is_empty() {
        return vec![];
    }
    let mut fields = Vec::new();
    for part in inner.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some(colon_pos) = part.find(':') {
            let field_name = part[..colon_pos].trim().trim_end_matches('?');
            let type_str = part[colon_pos + 1..].trim();
            fields.push((field_name.to_string(), type_str.to_string()));
        }
    }
    fields
}

fn render_spec_file(endpoint: &EndpointItem, py_name: &str) -> String {
    let mut imports = vec!["from __future__ import annotations".to_string()];
    let input_type = &endpoint.input_type_name;
    let builder_name = format!("build_{py_name}_spec");

    if is_void_input(input_type) {
        imports.push("from aptx_api_core import RequestSpec".to_string());

        let body = render_spec_fields(endpoint, false);
        format!(
            "{imports_block}\n\ndef {builder_name}() -> RequestSpec:\n    return RequestSpec(\n{body}    )\n",
            imports_block = imports.join("\n"),
            builder_name = builder_name,
            body = body,
        )
    } else if is_inline_input(input_type) {
        let inline_fields = parse_inline_fields(input_type);
        let params: Vec<String> = inline_fields
            .iter()
            .map(|(name, type_name)| format!("    *, {name}: {type_name}"))
            .collect();
        let sig = params.join(",\n");

        imports.push("from aptx_api_core import RequestSpec".to_string());

        let body = render_inline_spec_fields(endpoint);
        format!(
            "{imports_block}\n\ndef {builder_name}(\n{sig}\n) -> RequestSpec:\n    return RequestSpec(\n{body}    )\n",
            imports_block = imports.join("\n"),
            builder_name = builder_name,
            sig = sig,
            body = body,
        )
    } else {
        // Model input
        imports.push(format!(
            "from models.{} import {}",
            to_snake_module(input_type),
            input_type
        ));
        imports.push("from aptx_api_core import RequestSpec".to_string());

        let body = render_spec_fields(endpoint, true);
        format!(
            "{imports_block}\n\ndef {builder_name}(\n    input: {input_type}) -> RequestSpec:\n    return RequestSpec(\n{body}    )\n",
            imports_block = imports.join("\n"),
            builder_name = builder_name,
            input_type = input_type,
            body = body,
        )
    }
}

fn render_spec_fields(endpoint: &EndpointItem, has_model_input: bool) -> String {
    let mut fields = format!("        method=\"{}\",\n", endpoint.method);
    fields.push_str(&format!("        path=\"{}\",\n", endpoint.path));

    if has_model_input {
        if endpoint.request_body_field.is_some() {
            fields.push_str("        body=input.model_dump(by_alias=True),\n");
        }
        if !endpoint.query_fields.is_empty() {
            let keys: Vec<String> = endpoint
                .query_fields
                .iter()
                .map(|f| format!("\"{f}\": input.{f}"))
                .collect();
            fields.push_str(&format!("        query={{ {} }},\n", keys.join(", ")));
        }
        fields.push_str("        input=input,\n");
    } else {
        if !endpoint.query_fields.is_empty() {
            let keys: Vec<String> = endpoint
                .query_fields
                .iter()
                .map(|f| format!("\"{f}\": {f}"))
                .collect();
            fields.push_str(&format!("        query={{ {} }},\n", keys.join(", ")));
        }
    }

    fields
}

fn render_inline_spec_fields(endpoint: &EndpointItem) -> String {
    let mut fields = format!("        method=\"{}\",\n", endpoint.method);
    fields.push_str(&format!("        path=\"{}\",\n", endpoint.path));

    if !endpoint.query_fields.is_empty() {
        let keys: Vec<String> = endpoint
            .query_fields
            .iter()
            .map(|f| format!("\"{f}\": {f}"))
            .collect();
        fields.push_str(&format!("        query={{ {} }},\n", keys.join(", ")));
    }

    if endpoint.request_body_field.is_some() {
        fields.push_str("        body=body,\n");
    }

    fields
}

fn render_function_file(endpoint: &EndpointItem, py_name: &str) -> String {
    let mut imports = vec!["from __future__ import annotations".to_string()];
    let input_type = &endpoint.input_type_name;
    let output_type = &endpoint.output_type_name;
    let builder_name = format!("build_{py_name}_spec");

    imports.push("from aptx_api_core import get_api_client".to_string());

    let escaped_ns: Vec<String> = endpoint.namespace.iter().map(|s| escape_keyword(s)).collect();
    let spec_import = format!(
        "from spec.{}.{py_name}_spec import {builder_name}",
        escaped_ns.join("."),
    );
    imports.push(spec_import);

    let (signature, call_args) = if is_void_input(input_type) {
        (String::new(), String::new())
    } else if is_inline_input(input_type) {
        let inline_fields = parse_inline_fields(input_type);
        let params: Vec<String> = inline_fields
            .iter()
            .map(|(name, type_name)| format!("    *, {name}: {type_name}"))
            .collect();
        let args: Vec<String> = inline_fields.iter().map(|(name, _)| name.clone()).collect();
        (params.join(",\n"), args.join(", "))
    } else {
        imports.push(format!(
            "from models.{} import {}",
            to_snake_module(input_type),
            input_type
        ));
        (format!("    input: {input_type}"), "input".to_string())
    };

    let is_void_output = output_type == "void" || output_type == "None";
    if !is_void_output {
        imports.push(format!(
            "from models.{} import {}",
            to_snake_module(output_type),
            output_type
        ));
    }

    let return_type = if is_void_output {
        "None"
    } else {
        output_type.as_str()
    };

    let response_type_arg = if is_void_output {
        String::new()
    } else {
        format!(",\n        response_type={output_type}")
    };

    let call_expr = if is_void_input(input_type) {
        format!("{builder_name}()")
    } else {
        format!("{builder_name}({call_args})")
    };

    let sig_block = if signature.is_empty() {
        String::new()
    } else {
        format!("\n{signature},\n")
    };

    format!(
        "{imports_block}\n\nasync def {py_name}({sig_block}) -> {return_type}:\n    return await get_api_client().execute_async(\n        {call_expr}{response_type_arg}\n    )\n",
        imports_block = imports.join("\n"),
        sig_block = sig_block,
        return_type = return_type,
        call_expr = call_expr,
        response_type_arg = response_type_arg,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    fn make_endpoint(
        namespace: &[&str],
        operation_name: &str,
        method: &str,
        path: &str,
        input_type: &str,
        output_type: &str,
    ) -> EndpointItem {
        EndpointItem {
            namespace: namespace.iter().map(|s| s.to_string()).collect(),
            operation_name: operation_name.to_string(),
            export_name: operation_name.to_string(),
            builder_name: format!("build_{operation_name}_spec"),
            summary: None,
            method: method.to_string(),
            path: path.to_string(),
            input_type_name: input_type.to_string(),
            output_type_name: output_type.to_string(),
            request_body_field: None,
            query_fields: vec![],
            path_fields: vec![],
            has_request_options: false,
            deprecated: false,
            meta: IndexMap::new(),
        }
    }

    #[test]
    fn test_renderer_id() {
        assert_eq!(PythonFunctionsRenderer.id(), "python-functions");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("getUserInfo"), "get_user_info");
        assert_eq!(to_snake_case("getEducationalAPIClassGetInfo"), "get_educational_api_class_get_info");
        assert_eq!(to_snake_case("getUserID"), "get_user_id");
        assert_eq!(to_snake_case("parseHTMLString"), "parse_html_string");
    }

    #[test]
    fn test_find_common_prefix() {
        let endpoints = vec![
            make_endpoint(&["a"], "getEducationalAPIClassGetInfo", "GET", "/", "void", "void"),
            make_endpoint(&["a"], "postEducationalAPIClassAdd", "POST", "/", "void", "void"),
            make_endpoint(&["a"], "getEducationalAPIEnumsGetAll", "GET", "/", "void", "void"),
        ];
        assert_eq!(find_common_service_prefix(&endpoints), "EducationalAPI");
    }

    #[test]
    fn test_compute_py_name_strips_prefix_and_namespace() {
        let endpoints = vec![
            make_endpoint(&["class"], "getEducationalAPIClassGetInfo", "GET", "/", "void", "void"),
            make_endpoint(&["class"], "postEducationalAPIClassAdd", "POST", "/", "void", "void"),
            make_endpoint(&["enums"], "getEducationalAPIEnumsGetAllAuditStatusEnum", "GET", "/", "void", "void"),
            make_endpoint(&["class-schedule"], "getEducationalAPIClassScheduleGetInfo", "GET", "/", "void", "void"),
        ];
        let prefix = find_common_service_prefix(&endpoints);
        assert_eq!(compute_py_name(&endpoints[0], &prefix), "get_info");
        assert_eq!(compute_py_name(&endpoints[1], &prefix), "add");
        assert_eq!(compute_py_name(&endpoints[2], &prefix), "get_all_audit_status_enum");
        assert_eq!(compute_py_name(&endpoints[3], &prefix), "get_info");
    }

    #[test]
    fn test_no_prefix_with_single_endpoint() {
        let endpoints = vec![
            make_endpoint(&["a"], "getUser", "GET", "/", "void", "void"),
        ];
        assert_eq!(find_common_service_prefix(&endpoints), "");
    }

    #[test]
    fn test_void_input_spec() {
        let ep = make_endpoint(&["health"], "check", "GET", "/health", "void", "void");
        let content = render_spec_file(&ep, "check");
        assert!(content.contains("def build_check_spec() -> RequestSpec:"));
        assert!(content.contains("method=\"GET\""));
        assert!(content.contains("path=\"/health\""));
    }

    #[test]
    fn test_model_input_spec() {
        let mut ep = make_endpoint(
            &["users"], "get_user", "GET", "/users/{id}", "GetUserInput", "User",
        );
        ep.path_fields = vec!["id".to_string()];

        let content = render_spec_file(&ep, "get_user");
        assert!(content.contains("def build_get_user_spec("));
        assert!(content.contains("input: GetUserInput"));
        assert!(content.contains("from models.get_user_input import GetUserInput"));
    }

    #[test]
    fn test_model_input_function() {
        let ep = make_endpoint(
            &["users"], "get_user", "GET", "/users/{id}", "GetUserInput", "User",
        );
        let content = render_function_file(&ep, "get_user");
        assert!(content.contains("async def get_user("));
        assert!(content.contains("input: GetUserInput"));
        assert!(content.contains(") -> User:"));
        assert!(content.contains("response_type=User"));
        assert!(content.contains("from models.user import User"));
    }

    #[test]
    fn test_inline_input_spec_and_function() {
        let mut ep = make_endpoint(
            &["stored_file"],
            "upload_image",
            "POST",
            "/stored-file/upload",
            "{ StoreType: StoreType; body?: object }",
            "GuidResult",
        );
        ep.query_fields = vec!["StoreType".to_string()];
        ep.request_body_field = Some("body".to_string());

        let spec = render_spec_file(&ep, "upload_image");
        assert!(spec.contains("StoreType: StoreType"));
        assert!(spec.contains("query="));

        let func = render_function_file(&ep, "upload_image");
        assert!(func.contains("async def upload_image"));
        assert!(func.contains("-> GuidResult"));
        assert!(func.contains("response_type=GuidResult"));
    }

    #[test]
    fn test_post_with_body() {
        let mut ep = make_endpoint(
            &["users"], "add_user", "POST", "/users", "AddUserRequest", "User",
        );
        ep.request_body_field = Some("body".to_string());

        let spec = render_spec_file(&ep, "add_user");
        assert!(spec.contains("body=input.model_dump(by_alias=True)"));
    }

    #[test]
    fn test_void_output_function() {
        let ep = make_endpoint(&["health"], "check", "GET", "/health", "void", "void");
        let content = render_function_file(&ep, "check");
        assert!(content.contains(") -> None:"));
        assert!(!content.contains("response_type="));
    }
}
