//! Python functions renderer for swagger_gen.
//!
//! Generates spec files and function files that use aptx_api_core.

use std::collections::HashMap;

use swagger_gen::pipeline::{
    EndpointItem, GeneratorInput, PlannedFile, RenderOutput, Renderer,
    resolve_file_import_path, resolve_model_import_base, should_use_package_import,
};

/// Renderer that generates Python spec + function files.
#[derive(Default)]
pub struct PythonFunctionsRenderer;

impl Renderer for PythonFunctionsRenderer {
    fn id(&self) -> &'static str {
        "python-functions"
    }

    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String> {
        let resolved_names = resolve_final_py_names(&input.endpoints);
        let use_package = should_use_package_import(&input.model_import);
        let mut files = Vec::new();

        for (endpoint, py_name) in input.endpoints.iter().zip(resolved_names.iter()) {
            let spec_path = get_spec_file_path(endpoint, py_name);
            let function_path = get_function_file_path(endpoint, py_name);
            let spec_model_import_base =
                resolve_python_model_import_base(input, &spec_path, use_package);
            let function_model_import_base =
                resolve_python_model_import_base(input, &function_path, use_package);

            files.push(PlannedFile {
                path: spec_path,
                content: render_spec_file(endpoint, py_name, &spec_model_import_base),
            });
            files.push(PlannedFile {
                path: function_path.clone(),
                content: render_function_file(
                    endpoint,
                    py_name,
                    &function_path,
                    &function_model_import_base,
                ),
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

    let mut prefix_words = split_identifier_words(parts[0]);
    for part in &parts[1..] {
        let words = split_identifier_words(part);
        let common_len = prefix_words
            .iter()
            .zip(words.iter())
            .take_while(|(left, right)| left == right)
            .count();
        prefix_words.truncate(common_len);
        if prefix_words.is_empty() {
            return String::new();
        }
    }

    // Namespace stripping happens later. Do not let the shared prefix eat the namespace
    // segment itself or any action words that come after it.
    let namespace_words = split_identifier_words(&namespace_to_camel(&endpoints[0].namespace));
    if let Some(index) = find_word_sequence(&prefix_words, &namespace_words) {
        prefix_words.truncate(index);
        if prefix_words.is_empty() {
            return String::new();
        }
    }

    let prefix = prefix_words.join("");

    // Don't strip if any endpoint would be left with nothing
    if parts.iter().any(|p| p.strip_prefix(&prefix).unwrap_or("").is_empty()) {
        return String::new();
    }

    prefix
}

fn get_namespace_path(endpoint: &EndpointItem) -> String {
    endpoint
        .namespace
        .iter()
        .map(|s| escape_keyword(s))
        .collect::<Vec<_>>()
        .join("/")
}

fn compute_fallback_py_name(endpoint: &EndpointItem, common_prefix: &str) -> String {
    let method_end = endpoint.operation_name.find(|c: char| c.is_uppercase()).unwrap_or(0);
    let service_part = &endpoint.operation_name[method_end..];
    let after_service = service_part.strip_prefix(common_prefix).unwrap_or(service_part);

    if after_service.is_empty() {
        return to_snake_case(service_part);
    }

    to_snake_case(after_service)
}

fn resolve_namespace_common_prefixes(endpoints: &[EndpointItem]) -> HashMap<String, String> {
    let mut grouped: HashMap<String, Vec<&EndpointItem>> = HashMap::new();
    for endpoint in endpoints {
        grouped
            .entry(get_namespace_path(endpoint))
            .or_default()
            .push(endpoint);
    }

    grouped
        .into_iter()
        .map(|(namespace_path, group)| {
            let prefix = find_common_service_prefix(&group.into_iter().cloned().collect::<Vec<_>>());
            (namespace_path, prefix)
        })
        .collect()
}

fn resolve_final_py_names(endpoints: &[EndpointItem]) -> Vec<String> {
    let namespace_prefixes = resolve_namespace_common_prefixes(endpoints);
    let planned: Vec<(String, String, String)> = endpoints
        .iter()
        .map(|endpoint| {
            let namespace_path = get_namespace_path(endpoint);
            let common_prefix = namespace_prefixes
                .get(&namespace_path)
                .map(|s| s.as_str())
                .unwrap_or("");
            (
                namespace_path,
                compute_py_name(endpoint, common_prefix),
                compute_fallback_py_name(endpoint, common_prefix),
            )
        })
        .collect();

    let mut short_name_counts: HashMap<(String, String), usize> = HashMap::new();
    for (namespace_path, short_name, _) in &planned {
        *short_name_counts
            .entry((namespace_path.clone(), short_name.clone()))
            .or_insert(0) += 1;
    }

    planned
        .into_iter()
        .map(|(namespace_path, short_name, fallback_name)| {
            if short_name_counts
                .get(&(namespace_path, short_name.clone()))
                .copied()
                .unwrap_or(0)
                > 1
            {
                fallback_name
            } else {
                short_name
            }
        })
        .collect()
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

    // Strip namespace prefix, even when a service segment still appears before it.
    let ns_camel = namespace_to_camel(&endpoint.namespace);
    let action = if let Some(rest) = after_service.strip_prefix(&ns_camel) {
        rest
    } else if let Some(index) = after_service.find(&ns_camel) {
        &after_service[index + ns_camel.len()..]
    } else {
        after_service
    };

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
        .iter()
        .flat_map(|segment| segment.split(['-', '_']))
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect()
}

fn split_identifier_words(name: &str) -> Vec<String> {
    let chars: Vec<char> = name.chars().collect();
    if chars.is_empty() {
        return vec![];
    }

    let mut words = Vec::new();
    let mut current = String::new();

    for (index, ch) in chars.iter().enumerate() {
        let starts_new_word = if index == 0 {
            false
        } else if ch.is_uppercase() {
            let prev = chars[index - 1];
            let next_is_lower = chars.get(index + 1).is_some_and(|c| c.is_lowercase());
            prev.is_lowercase() || (prev.is_uppercase() && next_is_lower)
        } else {
            false
        };

        if starts_new_word && !current.is_empty() {
            words.push(current);
            current = String::new();
        }

        current.push(*ch);
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

fn find_word_sequence(haystack: &[String], needle: &[String]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }

    haystack.windows(needle.len()).position(|window| {
        window
            .iter()
            .zip(needle.iter())
            .all(|(left, right)| left == right)
    })
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
    name.to_string()
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

fn normalize_python_package_path(import_path: &str) -> String {
    import_path
        .replace('\\', ".")
        .replace('/', ".")
        .trim_matches('.')
        .to_string()
}

fn to_python_relative_import_path(import_path: &str) -> String {
    let normalized = import_path.replace('\\', "/");
    let mut rest = normalized.as_str();
    let mut parent_levels = 0usize;

    while let Some(next) = rest.strip_prefix("../") {
        parent_levels += 1;
        rest = next;
    }

    if rest == ".." {
        parent_levels += 1;
        rest = "";
    }

    if let Some(next) = rest.strip_prefix("./") {
        rest = next;
    } else if rest == "." {
        rest = "";
    }

    let prefix = ".".repeat(parent_levels + 1);
    let suffix = rest.trim_matches('/').replace('/', ".");

    if suffix.is_empty() {
        prefix
    } else {
        format!("{prefix}{suffix}")
    }
}

fn resolve_python_module_import(from_file_path: &str, to_file_path: &str) -> String {
    to_python_relative_import_path(&resolve_file_import_path(from_file_path, to_file_path))
}

fn resolve_python_model_import_base(
    input: &GeneratorInput,
    generated_file_path: &str,
    use_package: bool,
) -> String {
    if let Some(config) = &input.model_import {
        let base = resolve_model_import_base(input, generated_file_path);
        if use_package || config.import_type == "package" {
            return normalize_python_package_path(&base);
        }
        return to_python_relative_import_path(&base);
    }

    to_python_relative_import_path(&resolve_file_import_path(generated_file_path, "models"))
}

fn render_spec_file(endpoint: &EndpointItem, py_name: &str, model_import_base: &str) -> String {
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
            "from {model_import_base}.{} import {}",
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

fn render_function_file(
    endpoint: &EndpointItem,
    py_name: &str,
    current_file_path: &str,
    model_import_base: &str,
) -> String {
    let mut imports = vec!["from __future__ import annotations".to_string()];
    let input_type = &endpoint.input_type_name;
    let output_type = &endpoint.output_type_name;
    let builder_name = format!("build_{py_name}_spec");

    imports.push("from aptx_api_core import get_api_client".to_string());

    let spec_file_path = get_spec_file_path(endpoint, py_name);
    let spec_import = format!(
        "from {} import {builder_name}",
        resolve_python_module_import(current_file_path, &spec_file_path),
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
            "from {model_import_base}.{} import {}",
            to_snake_module(input_type),
            input_type
        ));
        (format!("    input: {input_type}"), "input".to_string())
    };

    let is_void_output = output_type == "void" || output_type == "None";
    if !is_void_output {
        imports.push(format!(
            "from {model_import_base}.{} import {}",
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
        "{imports_block}\n\ndef {py_name}({sig_block}) -> {return_type}:\n    return get_api_client().execute(\n        {call_expr}{response_type_arg}\n    )\n",
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
    use swagger_gen::pipeline::{GeneratorInput, ModelImportConfig, ProjectContext};

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

    fn make_generator_input_with_model_import(
        endpoints: Vec<EndpointItem>,
        model_import: Option<ModelImportConfig>,
        output_root: Option<String>,
    ) -> GeneratorInput {
        GeneratorInput {
            project: ProjectContext {
                package_name: "test".to_string(),
                api_base_path: None,
                terminals: vec![],
                retry_ownership: None,
            },
            endpoints,
            model_import,
            client_import: None,
            output_root,
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
        let content = render_spec_file(&ep, "check", "...models");
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

        let content = render_spec_file(&ep, "get_user", "...models");
        assert!(content.contains("def build_get_user_spec("));
        assert!(content.contains("input: GetUserInput"));
        assert!(content.contains("from ...models.GetUserInput import GetUserInput"));
    }

    #[test]
    fn test_model_input_function() {
        let ep = make_endpoint(
            &["users"], "get_user", "GET", "/users/{id}", "GetUserInput", "User",
        );
        let content = render_function_file(
            &ep,
            "get_user",
            "functions/users/get_user.py",
            "...models",
        );
        assert!(content.contains("def get_user("));
        assert!(!content.contains("async def get_user("));
        assert!(content.contains("input: GetUserInput"));
        assert!(content.contains(") -> User:"));
        assert!(content.contains("response_type=User"));
        assert!(content.contains("get_api_client().execute("));
        assert!(content.contains("from ...models.User import User"));
    }

    #[test]
    fn test_model_imports_use_pascal_case_modules() {
        let ep = make_endpoint(
            &["users"], "get_user", "GET", "/users/{id}", "GetUserInput", "User",
        );

        let spec = render_spec_file(&ep, "get_user", "...models");
        assert!(spec.contains("from ...models.GetUserInput import GetUserInput"));

        let function = render_function_file(
            &ep,
            "get_user",
            "functions/users/get_user.py",
            "...models",
        );
        assert!(function.contains("from ...models.GetUserInput import GetUserInput"));
        assert!(function.contains("from ...models.User import User"));
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

        let spec = render_spec_file(&ep, "upload_image", "...models");
        assert!(spec.contains("StoreType: StoreType"));
        assert!(spec.contains("query="));

        let func = render_function_file(
            &ep,
            "upload_image",
            "functions/stored_file/upload_image.py",
            "...models",
        );
        assert!(func.contains("def upload_image"));
        assert!(!func.contains("async def upload_image"));
        assert!(func.contains("-> GuidResult"));
        assert!(func.contains("response_type=GuidResult"));
        assert!(func.contains("get_api_client().execute("));
    }

    #[test]
    fn test_post_with_body() {
        let mut ep = make_endpoint(
            &["users"], "add_user", "POST", "/users", "AddUserRequest", "User",
        );
        ep.request_body_field = Some("body".to_string());

        let spec = render_spec_file(&ep, "add_user", "...models");
        assert!(spec.contains("body=input.model_dump(by_alias=True)"));
    }

    #[test]
    fn test_void_output_function() {
        let ep = make_endpoint(&["health"], "check", "GET", "/health", "void", "void");
        let content = render_function_file(
            &ep,
            "check",
            "functions/health/check.py",
            "...models",
        );
        assert!(content.contains(") -> None:"));
        assert!(!content.contains("response_type="));
    }

    #[test]
    fn test_common_prefix_does_not_strip_partial_word() {
        let endpoints = vec![
            make_endpoint(
                &["action_authority"],
                "getActionAuthorityAPIList",
                "GET",
                "/action-authority/list",
                "void",
                "void",
            ),
            make_endpoint(
                &["action_assignment"],
                "getActionAssignmentAPIList",
                "GET",
                "/action-assignment/list",
                "void",
                "void",
            ),
        ];

        let prefix = find_common_service_prefix(&endpoints);
        assert_eq!(prefix, "Action");
        assert_eq!(compute_py_name(&endpoints[0], &prefix), "authority_api_list");
    }

    #[test]
    fn test_compute_py_name_strips_underscore_namespace_prefix() {
        let endpoints = vec![
            make_endpoint(
                &["action_authority"],
                "getEducationalAPIActionAuthorityGetInfo",
                "GET",
                "/action-authority/info",
                "void",
                "void",
            ),
            make_endpoint(
                &["users"],
                "getEducationalAPIUsersGetInfo",
                "GET",
                "/users/info",
                "void",
                "void",
            ),
        ];

        let prefix = find_common_service_prefix(&endpoints);
        assert_eq!(prefix, "EducationalAPI");
        assert_eq!(compute_py_name(&endpoints[0], &prefix), "get_info");
    }

    #[test]
    fn test_compute_fallback_py_name_keeps_longer_service_name() {
        let endpoints = vec![
            make_endpoint(
                &["user"],
                "postAuthorityAPIUserAdd",
                "POST",
                "/AuthorityAPI/User/Add",
                "void",
                "void",
            ),
            make_endpoint(
                &["user"],
                "postAuthorityAPIUserUserAdd",
                "POST",
                "/AuthorityAPI/User/UserAdd",
                "void",
                "void",
            ),
        ];

        let prefix = find_common_service_prefix(&endpoints);
        assert_eq!(compute_fallback_py_name(&endpoints[0], &prefix), "user_add");
        assert_eq!(compute_fallback_py_name(&endpoints[1], &prefix), "user_user_add");
    }

    #[test]
    fn test_render_uses_short_name_inside_namespace_directory() {
        let input = make_generator_input(vec![
            make_endpoint(
                &["action_authority"],
                "postAuthorityAPIActionAuthorityAdd",
                "POST",
                "/AuthorityAPI/ActionAuthority/Add",
                "AddActionAuthorityRequestModel",
                "GuidResultModel",
            ),
            make_endpoint(
                &["role"],
                "postAuthorityAPIRoleAdd",
                "POST",
                "/AuthorityAPI/Role/Add",
                "AddRoleRequestModel",
                "GuidResultModel",
            ),
        ]);

        let output = PythonFunctionsRenderer.render(&input).unwrap();
        assert!(output.files.iter().any(|f| f.path == "functions/action_authority/add.py"));
        assert!(output.files.iter().any(|f| f.path == "spec/action_authority/add_spec.py"));
        assert!(output.files.iter().any(|f| f.path == "functions/role/add.py"));

        let action_function = output
            .files
            .iter()
            .find(|f| f.path == "functions/action_authority/add.py")
            .unwrap();
        assert!(action_function.content.contains("from ...spec.action_authority.add_spec import build_add_spec"));
        assert!(action_function.content.contains("def add("));
        assert!(!action_function.content.contains("async def add("));
    }

    #[test]
    fn test_name_collision_falls_back_to_long_name_within_namespace() {
        let input = make_generator_input(vec![
            make_endpoint(
                &["user"],
                "postAuthorityAPIUserAdd",
                "POST",
                "/AuthorityAPI/User/Add",
                "void",
                "void",
            ),
            make_endpoint(
                &["user"],
                "postAuthorityAPIUserUserAdd",
                "POST",
                "/AuthorityAPI/User/UserAdd",
                "void",
                "void",
            ),
        ]);

        let output = PythonFunctionsRenderer.render(&input).unwrap();
        assert!(output.files.iter().any(|f| f.path == "functions/user/add.py"));
        assert!(output.files.iter().any(|f| f.path == "functions/user/user_add.py"));
        assert!(output.files.iter().all(|f| f.path != "functions/user/add.py" || !f.content.contains("build_user_add_spec")));
    }

    #[test]
    fn test_render_uses_namespace_local_common_prefix_when_global_prefix_is_empty() {
        let input = make_generator_input(vec![
            make_endpoint(
                &["action-authority"],
                "postAuthorityAPIActionAuthorityAdd",
                "POST",
                "/AuthorityAPI/ActionAuthority/Add",
                "AddActionAuthorityRequestModel",
                "GuidResultModel",
            ),
            make_endpoint(
                &["role"],
                "postAuthorityAPIRoleAdd",
                "POST",
                "/AuthorityAPI/Role/Add",
                "AddRoleRequestModel",
                "GuidResultModel",
            ),
            make_endpoint(
                &["wechat"],
                "getApiWechat",
                "GET",
                "/ApiWechat",
                "void",
                "void",
            ),
        ]);

        let output = PythonFunctionsRenderer.render(&input).unwrap();
        assert!(output.files.iter().any(|f| f.path == "functions/action_authority/add.py"));
        assert!(output.files.iter().any(|f| f.path == "functions/role/add.py"));
    }

    #[test]
    fn test_render_keeps_full_action_name_when_namespace_local_prefix_would_over_shorten() {
        let input = make_generator_input(vec![
            make_endpoint(
                &["user"],
                "getAuthorityAPIUserGetLoginUserInfo",
                "GET",
                "/AuthorityAPI/User/GetLoginUserInfo",
                "void",
                "void",
            ),
            make_endpoint(
                &["user"],
                "getAuthorityAPIUserGetLoginUserPermissions",
                "GET",
                "/AuthorityAPI/User/GetLoginUserPermissions",
                "void",
                "void",
            ),
        ]);

        let output = PythonFunctionsRenderer.render(&input).unwrap();
        assert!(output
            .files
            .iter()
            .any(|f| f.path == "functions/user/get_login_user_info.py"));
        assert!(output
            .files
            .iter()
            .any(|f| f.path == "spec/user/get_login_user_info_spec.py"));
        assert!(output
            .files
            .iter()
            .any(|f| f.path == "functions/user/get_login_user_permissions.py"));
        assert!(!output.files.iter().any(|f| f.path == "functions/user/info.py"));
        assert!(!output
            .files
            .iter()
            .any(|f| f.path == "spec/user/info_spec.py"));
    }

    #[test]
    fn test_render_uses_relative_package_imports_for_models_and_specs() {
        let output_root = std::env::current_dir()
            .expect("cwd")
            .join("target/python-relative-imports");
        let output_root_string = output_root.to_string_lossy().to_string();
        let model_dir = output_root.join("models").to_string_lossy().to_string();

        let input = make_generator_input_with_model_import(
            vec![make_endpoint(
                &["users"],
                "getUser",
                "GET",
                "/users/{id}",
                "GetUserInput",
                "User",
            )],
            Some(ModelImportConfig {
                import_type: "relative".to_string(),
                package_path: None,
                relative_path: None,
                original_path: Some(model_dir),
            }),
            Some(output_root_string),
        );

        let output = PythonFunctionsRenderer.render(&input).unwrap();
        let spec = output
            .files
            .iter()
            .find(|f| f.path.starts_with("spec/users/") && f.path.ends_with("_spec.py"))
            .expect("spec file");
        let function = output
            .files
            .iter()
            .find(|f| f.path.starts_with("functions/users/") && f.path.ends_with(".py"))
            .expect("function file");

        assert!(spec.content.contains("from ...models.GetUserInput import GetUserInput"));
        assert!(function.content.contains("from ...models.GetUserInput import GetUserInput"));
        assert!(function.content.contains("from ...models.User import User"));
        assert!(function.content.contains("from ...spec.users."));
    }

    #[test]
    fn test_render_uses_package_imports_when_model_mode_is_package() {
        let input = make_generator_input_with_model_import(
            vec![make_endpoint(
                &["users"],
                "getUser",
                "GET",
                "/users/{id}",
                "GetUserInput",
                "User",
            )],
            Some(ModelImportConfig {
                import_type: "package".to_string(),
                package_path: Some("my_app.generated.models".to_string()),
                relative_path: None,
                original_path: Some("my_app.generated.models".to_string()),
            }),
            Some("target/python-package-imports".to_string()),
        );

        let output = PythonFunctionsRenderer.render(&input).unwrap();
        let spec = output
            .files
            .iter()
            .find(|f| f.path.starts_with("spec/users/") && f.path.ends_with("_spec.py"))
            .expect("spec file");
        let function = output
            .files
            .iter()
            .find(|f| f.path.starts_with("functions/users/") && f.path.ends_with(".py"))
            .expect("function file");

        assert!(spec.content.contains("from my_app.generated.models.GetUserInput import GetUserInput"));
        assert!(function.content.contains("from my_app.generated.models.GetUserInput import GetUserInput"));
        assert!(function.content.contains("from my_app.generated.models.User import User"));
    }
}
