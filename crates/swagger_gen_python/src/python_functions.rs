//! Python functions renderer for swagger_gen.
//!
//! Generates spec files and function files that use aptx_api_core.

use std::collections::HashMap;

use swagger_gen::pipeline::{
    EndpointItem, EndpointParameter, GeneratorInput, PlannedFile, RenderOutput, Renderer,
    resolve_file_import_path, resolve_model_import_base, should_use_package_import,
};

/// Renderer that generates Python spec + function files.
#[derive(Default)]
pub struct PythonFunctionsRenderer;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedPyName {
    file_stem: String,
    export_name: String,
    builder_name: String,
}

#[derive(Debug, Clone)]
struct PlannedPyName {
    namespace_path: String,
    short_name: String,
    fallback_name: String,
    fallback_export_name: String,
}

impl Renderer for PythonFunctionsRenderer {
    fn id(&self) -> &'static str {
        "python-functions"
    }

    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String> {
        let resolved_names = resolve_final_py_names(&input.endpoints);
        let use_package = should_use_package_import(&input.model_import);
        let mut files = Vec::new();

        for (endpoint, resolved_name) in input.endpoints.iter().zip(resolved_names.iter()) {
            let spec_path = get_spec_file_path(endpoint, resolved_name);
            let function_path = get_function_file_path(endpoint, resolved_name);
            let spec_model_import_base =
                resolve_python_model_import_base(input, &spec_path, use_package);
            let function_model_import_base =
                resolve_python_model_import_base(input, &function_path, use_package);

            files.push(PlannedFile {
                path: spec_path,
                content: render_spec_file(endpoint, resolved_name, &spec_model_import_base),
            });
            files.push(PlannedFile {
                path: function_path.clone(),
                content: render_function_file(
                    endpoint,
                    resolved_name,
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

fn get_spec_file_path(endpoint: &EndpointItem, resolved_name: &ResolvedPyName) -> String {
    let namespace: Vec<String> = endpoint
        .namespace
        .iter()
        .map(|s| escape_keyword(s))
        .collect();
    let namespace = namespace.join("/");
    format!("spec/{namespace}/{}_spec.py", resolved_name.file_stem)
}

fn get_function_file_path(endpoint: &EndpointItem, resolved_name: &ResolvedPyName) -> String {
    let namespace: Vec<String> = endpoint
        .namespace
        .iter()
        .map(|s| escape_keyword(s))
        .collect();
    let namespace = namespace.join("/");
    format!("functions/{namespace}/{}.py", resolved_name.file_stem)
}

const PYTHON_KEYWORDS: &[&str] = &[
    "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class", "continue",
    "def", "del", "elif", "else", "except", "finally", "for", "from", "global", "if", "import",
    "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try", "while",
    "with", "yield",
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
    if parts
        .iter()
        .any(|p| p.strip_prefix(&prefix).unwrap_or("").is_empty())
    {
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
    let method_end = endpoint
        .operation_name
        .find(|c: char| c.is_uppercase())
        .unwrap_or(0);
    let service_part = &endpoint.operation_name[method_end..];
    let after_service = service_part
        .strip_prefix(common_prefix)
        .unwrap_or(service_part);

    if after_service.is_empty() {
        return to_snake_case(service_part);
    }

    to_snake_case(after_service)
}

fn compute_fallback_py_export_name(endpoint: &EndpointItem, common_prefix: &str) -> String {
    let fallback = if endpoint.export_name.trim().is_empty() {
        compute_fallback_py_name(endpoint, common_prefix)
    } else {
        to_snake_case(&endpoint.export_name)
    };

    sanitize_python_identifier(&fallback)
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
            let prefix =
                find_common_service_prefix(&group.into_iter().cloned().collect::<Vec<_>>());
            (namespace_path, prefix)
        })
        .collect()
}

fn resolve_final_py_names(endpoints: &[EndpointItem]) -> Vec<ResolvedPyName> {
    let namespace_prefixes = resolve_namespace_common_prefixes(endpoints);
    let planned: Vec<PlannedPyName> = endpoints
        .iter()
        .map(|endpoint| {
            let namespace_path = get_namespace_path(endpoint);
            let common_prefix = namespace_prefixes
                .get(&namespace_path)
                .map(|s| s.as_str())
                .unwrap_or("");
            PlannedPyName {
                namespace_path,
                short_name: compute_py_name(endpoint, common_prefix),
                fallback_name: compute_fallback_py_name(endpoint, common_prefix),
                fallback_export_name: compute_fallback_py_export_name(endpoint, common_prefix),
            }
        })
        .collect();

    let file_stems = resolve_local_py_names(&planned);
    let export_names = resolve_global_py_export_names(&planned);

    file_stems
        .into_iter()
        .zip(export_names)
        .map(|(file_stem, export_name)| ResolvedPyName {
            file_stem,
            builder_name: format!("build_{export_name}_spec"),
            export_name,
        })
        .collect()
}

fn resolve_local_py_names(planned: &[PlannedPyName]) -> Vec<String> {
    let mut short_name_counts: HashMap<(String, String), usize> = HashMap::new();
    for item in planned {
        *short_name_counts
            .entry((item.namespace_path.clone(), item.short_name.clone()))
            .or_insert(0) += 1;
    }

    planned
        .iter()
        .map(|item| {
            if short_name_counts
                .get(&(item.namespace_path.clone(), item.short_name.clone()))
                .copied()
                .unwrap_or(0)
                > 1
            {
                item.fallback_name.clone()
            } else {
                item.short_name.clone()
            }
        })
        .collect()
}

fn resolve_global_py_export_names(planned: &[PlannedPyName]) -> Vec<String> {
    let mut used_counts: HashMap<String, usize> = HashMap::new();

    planned
        .iter()
        .map(|item| {
            let mut final_name = item.fallback_export_name.clone();

            if used_counts.get(&final_name).copied().unwrap_or(0) > 0 {
                let mut serial = 2usize;
                let mut candidate = format!("{final_name}_{serial}");
                while used_counts.get(&candidate).copied().unwrap_or(0) > 0 {
                    serial += 1;
                    candidate = format!("{final_name}_{serial}");
                }
                final_name = candidate;
            }

            *used_counts.entry(final_name.clone()).or_insert(0) += 1;
            final_name
        })
        .collect()
}

/// Compute the Python operation name:
/// 1. Skip the HTTP method prefix (get/post/put/delete)
/// 2. Strip the common service prefix (e.g. "EducationalAPI")
/// 3. Strip the namespace prefix (e.g. "Class" for namespace ["class"])
/// 4. Convert remaining action to snake_case
fn compute_py_name(endpoint: &EndpointItem, common_prefix: &str) -> String {
    let method_end = endpoint
        .operation_name
        .find(|c: char| c.is_uppercase())
        .unwrap_or(0);
    let service_part = &endpoint.operation_name[method_end..]; // skip HTTP method

    // Strip common service prefix
    let after_service = service_part
        .strip_prefix(common_prefix)
        .unwrap_or(service_part);

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

#[derive(Clone)]
struct RenderedExtraParam {
    original_name: String,
    python_name: String,
    annotation: String,
    required: bool,
    imports: Vec<String>,
}

struct RenderedPythonType {
    annotation: String,
    imports: Vec<String>,
    runtime_type: Option<String>,
}

fn sanitize_python_identifier(name: &str) -> String {
    let mut result = String::new();

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            result.push(ch);
        } else {
            result.push('_');
        }
    }

    if result.is_empty() {
        result.push_str("param");
    }

    if result.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        result.insert(0, '_');
    }

    escape_keyword(&result)
}

fn render_python_annotation(type_name: &str, model_import_base: &str) -> (String, Vec<String>) {
    let trimmed = type_name.trim();

    if let Some(inner) = trimmed
        .strip_prefix("Array<")
        .and_then(|rest| rest.strip_suffix('>'))
    {
        let (inner_annotation, imports) = render_python_annotation(inner, model_import_base);
        return (format!("list[{inner_annotation}]"), imports);
    }

    match trimmed {
        "string" => ("str".to_string(), vec![]),
        "number" => ("float".to_string(), vec![]),
        "boolean" => ("bool".to_string(), vec![]),
        "object" => (
            "dict[str, Any]".to_string(),
            vec!["from typing import Any".to_string()],
        ),
        _ if trimmed
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_') =>
        {
            (
                trimmed.to_string(),
                vec![format!(
                    "from {model_import_base}.{} import {}",
                    to_snake_module(trimmed),
                    trimmed
                )],
            )
        }
        _ => (
            "Any".to_string(),
            vec!["from typing import Any".to_string()],
        ),
    }
}

fn is_python_model_reference(type_name: &str) -> bool {
    let trimmed = type_name.trim();

    !matches!(trimmed, "string" | "number" | "boolean" | "object")
        && trimmed
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        && !trimmed.contains(['<', '>', '|', '[', ']', '{', '}', ';'])
}

fn render_python_type_usage(type_name: &str, model_import_base: &str) -> RenderedPythonType {
    let (annotation, imports) = render_python_annotation(type_name, model_import_base);
    let runtime_type = is_python_model_reference(type_name).then(|| type_name.trim().to_string());

    RenderedPythonType {
        annotation,
        imports,
        runtime_type,
    }
}

fn render_request_body_value(value_expr: &str, type_name: &str) -> String {
    render_request_body_value_with_depth(value_expr, type_name, 0)
}

fn render_request_body_value_with_depth(value_expr: &str, type_name: &str, depth: usize) -> String {
    let trimmed = type_name.trim();

    if let Some(inner) = trimmed
        .strip_prefix("Array<")
        .and_then(|rest| rest.strip_suffix('>'))
    {
        let item_var = format!("item_{depth}");
        let rendered_inner = render_request_body_value_with_depth(&item_var, inner, depth + 1);
        if rendered_inner == item_var {
            return value_expr.to_string();
        }
        return format!("[{rendered_inner} for {item_var} in {value_expr}]");
    }

    if matches!(trimmed, "string" | "number" | "boolean" | "object") {
        return value_expr.to_string();
    }

    if is_python_model_reference(trimmed) {
        return format!("{value_expr}.model_dump(by_alias=True, exclude_none=True)");
    }

    value_expr.to_string()
}

fn render_extra_param(
    parameter: &EndpointParameter,
    model_import_base: &str,
) -> RenderedExtraParam {
    let (annotation, imports) = render_python_annotation(&parameter.type_name, model_import_base);

    RenderedExtraParam {
        original_name: parameter.name.clone(),
        python_name: sanitize_python_identifier(&parameter.name),
        annotation,
        required: parameter.required,
        imports,
    }
}

fn collect_extra_params(
    endpoint: &EndpointItem,
    model_import_base: &str,
) -> Vec<RenderedExtraParam> {
    endpoint
        .path_params
        .iter()
        .chain(endpoint.query_params.iter())
        .map(|parameter| render_extra_param(parameter, model_import_base))
        .collect()
}

fn find_rendered_param<'a>(
    params: &'a [RenderedExtraParam],
    original_name: &str,
) -> Option<&'a RenderedExtraParam> {
    params
        .iter()
        .find(|parameter| parameter.original_name == original_name)
}

fn render_extra_param_signature(parameter: &RenderedExtraParam) -> String {
    if parameter.required {
        format!("    {}: {}", parameter.python_name, parameter.annotation)
    } else {
        format!(
            "    {}: {} | None = None",
            parameter.python_name, parameter.annotation
        )
    }
}

fn render_signature_block(
    primary_param: Option<String>,
    extra_params: &[RenderedExtraParam],
) -> String {
    let mut lines = Vec::new();

    if let Some(primary_param) = primary_param {
        lines.push(format!("    {primary_param}"));
    }

    if !extra_params.is_empty() {
        lines.push("    *".to_string());
        lines.extend(extra_params.iter().map(render_extra_param_signature));
    }

    if lines.is_empty() {
        String::new()
    } else {
        format!("\n{},\n", lines.join(",\n"))
    }
}

fn render_builder_call(
    builder_name: &str,
    primary_arg: Option<&str>,
    extra_params: &[RenderedExtraParam],
) -> String {
    let mut args = Vec::new();

    if let Some(primary_arg) = primary_arg {
        args.push(primary_arg.to_string());
    }

    args.extend(
        extra_params
            .iter()
            .map(|parameter| format!("{}={}", parameter.original_name, parameter.python_name)),
    );

    if args.is_empty() {
        format!("{builder_name}()")
    } else {
        format!("{builder_name}({})", args.join(", "))
    }
}

fn resolve_request_value(
    field_name: &str,
    extra_params: &[RenderedExtraParam],
    has_model_input: bool,
) -> String {
    if let Some(parameter) = find_rendered_param(extra_params, field_name) {
        parameter.python_name.clone()
    } else if has_model_input {
        format!("input.{field_name}")
    } else {
        sanitize_python_identifier(field_name)
    }
}

fn render_path_input_assignment(
    endpoint: &EndpointItem,
    extra_params: &[RenderedExtraParam],
    has_model_input: bool,
) -> Option<String> {
    if endpoint.path_fields.is_empty() {
        return has_model_input.then(|| "        input=input,\n".to_string());
    }

    let fields = endpoint
        .path_fields
        .iter()
        .map(|field| {
            format!(
                "{field}={}",
                resolve_request_value(field, extra_params, has_model_input)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    Some(format!("        input=SimpleNamespace({fields}),\n"))
}

fn dedupe_lines(lines: Vec<String>) -> Vec<String> {
    let mut deduped = Vec::new();

    for line in lines {
        if !deduped.contains(&line) {
            deduped.push(line);
        }
    }

    deduped
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

fn render_spec_file(
    endpoint: &EndpointItem,
    resolved_name: &ResolvedPyName,
    model_import_base: &str,
) -> String {
    let mut imports = vec!["from __future__ import annotations".to_string()];
    let input_type = &endpoint.input_type_name;
    let builder_name = &resolved_name.builder_name;
    let extra_params = collect_extra_params(endpoint, model_import_base);
    let primary_input = (!is_void_input(input_type) && !is_inline_input(input_type))
        .then(|| render_python_type_usage(input_type, model_import_base));

    if !endpoint.path_fields.is_empty() {
        imports.push("from types import SimpleNamespace".to_string());
    }
    for parameter in &extra_params {
        imports.extend(parameter.imports.iter().cloned());
    }
    if let Some(primary_input) = &primary_input {
        imports.extend(primary_input.imports.iter().cloned());
    }

    if is_void_input(input_type) {
        imports.push("from aptx_api_core import RequestSpec".to_string());
        imports = dedupe_lines(imports);
        let sig_block = render_signature_block(None, &extra_params);
        let body = render_spec_fields(endpoint, false, &extra_params, input_type);
        format!(
            "{imports_block}\n\ndef {builder_name}({sig_block}) -> RequestSpec:\n    return RequestSpec(\n{body}    )\n",
            imports_block = imports.join("\n"),
            builder_name = builder_name,
            sig_block = sig_block,
            body = body,
        )
    } else if is_inline_input(input_type) {
        let inline_fields = parse_inline_fields(input_type);
        let rendered_inline_fields: Vec<(String, RenderedPythonType)> = inline_fields
            .iter()
            .map(|(name, type_name)| {
                (
                    name.clone(),
                    render_python_type_usage(type_name, model_import_base),
                )
            })
            .collect();
        for (_, rendered_type) in &rendered_inline_fields {
            imports.extend(rendered_type.imports.iter().cloned());
        }
        let params: Vec<String> = rendered_inline_fields
            .iter()
            .map(|(name, rendered_type)| format!("    *, {name}: {}", rendered_type.annotation))
            .collect();
        let sig = params.join(",\n");

        imports.push("from aptx_api_core import RequestSpec".to_string());
        imports = dedupe_lines(imports);

        let body = render_inline_spec_fields(endpoint, &inline_fields);
        format!(
            "{imports_block}\n\ndef {builder_name}(\n{sig}\n) -> RequestSpec:\n    return RequestSpec(\n{body}    )\n",
            imports_block = imports.join("\n"),
            builder_name = builder_name,
            sig = sig,
            body = body,
        )
    } else {
        imports.push("from aptx_api_core import RequestSpec".to_string());
        imports = dedupe_lines(imports);
        let primary_input = primary_input.expect("primary input type");
        let sig_block = render_signature_block(
            Some(format!("input: {}", primary_input.annotation)),
            &extra_params,
        );
        let body = render_spec_fields(endpoint, true, &extra_params, input_type);
        format!(
            "{imports_block}\n\ndef {builder_name}({sig_block}) -> RequestSpec:\n    return RequestSpec(\n{body}    )\n",
            imports_block = imports.join("\n"),
            builder_name = builder_name,
            sig_block = sig_block,
            body = body,
        )
    }
}

fn render_spec_fields(
    endpoint: &EndpointItem,
    has_model_input: bool,
    extra_params: &[RenderedExtraParam],
    input_type: &str,
) -> String {
    let mut fields = format!("        method=\"{}\",\n", endpoint.method);
    fields.push_str(&format!("        path=\"{}\",\n", endpoint.path));

    if has_model_input {
        if endpoint.request_body_field.is_some() {
            fields.push_str(&format!(
                "        body={},\n",
                render_request_body_value("input", input_type)
            ));
        }
    } else {
        if endpoint.request_body_field.is_some() {
            fields.push_str("        body=body,\n");
        }
    }

    if !endpoint.query_fields.is_empty() {
        let keys: Vec<String> = endpoint
            .query_fields
            .iter()
            .map(|field| {
                format!(
                    "\"{field}\": {}",
                    resolve_request_value(field, extra_params, has_model_input)
                )
            })
            .collect();
        fields.push_str(&format!("        query={{ {} }},\n", keys.join(", ")));
    }

    if let Some(input_assignment) =
        render_path_input_assignment(endpoint, extra_params, has_model_input)
    {
        fields.push_str(&input_assignment);
    }

    fields
}

fn render_inline_spec_fields(
    endpoint: &EndpointItem,
    inline_fields: &[(String, String)],
) -> String {
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
        let body_type = inline_fields
            .iter()
            .find(|(name, _)| name == "body")
            .map(|(_, type_name)| type_name.as_str())
            .unwrap_or("object");
        fields.push_str(&format!(
            "        body={},\n",
            render_request_body_value("body", body_type)
        ));
    }

    fields
}

fn render_function_file(
    endpoint: &EndpointItem,
    resolved_name: &ResolvedPyName,
    current_file_path: &str,
    model_import_base: &str,
) -> String {
    let mut imports = vec!["from __future__ import annotations".to_string()];
    let input_type = &endpoint.input_type_name;
    let output_type = &endpoint.output_type_name;
    let builder_name = &resolved_name.builder_name;
    let extra_params = collect_extra_params(endpoint, model_import_base);
    let primary_input = (!is_void_input(input_type) && !is_inline_input(input_type))
        .then(|| render_python_type_usage(input_type, model_import_base));
    let rendered_output = (output_type != "void" && output_type != "None")
        .then(|| render_python_type_usage(output_type, model_import_base));

    imports.push("from aptx_api_core import get_api_client".to_string());

    let spec_file_path = get_spec_file_path(endpoint, resolved_name);
    let spec_import = format!(
        "from {} import {builder_name}",
        resolve_python_module_import(current_file_path, &spec_file_path),
    );
    imports.push(spec_import);
    for parameter in &extra_params {
        imports.extend(parameter.imports.iter().cloned());
    }
    if let Some(primary_input) = &primary_input {
        imports.extend(primary_input.imports.iter().cloned());
    }
    if let Some(rendered_output) = &rendered_output {
        imports.extend(rendered_output.imports.iter().cloned());
    }

    let (signature, call_args) = if is_void_input(input_type) {
        (
            render_signature_block(None, &extra_params),
            render_builder_call(&builder_name, None, &extra_params),
        )
    } else if is_inline_input(input_type) {
        let inline_fields = parse_inline_fields(input_type);
        let rendered_inline_fields: Vec<(String, RenderedPythonType)> = inline_fields
            .iter()
            .map(|(name, type_name)| {
                (
                    name.clone(),
                    render_python_type_usage(type_name, model_import_base),
                )
            })
            .collect();
        for (_, rendered_type) in &rendered_inline_fields {
            imports.extend(rendered_type.imports.iter().cloned());
        }
        let params: Vec<String> = rendered_inline_fields
            .iter()
            .map(|(name, rendered_type)| format!("    *, {name}: {}", rendered_type.annotation))
            .collect();
        let args: Vec<String> = inline_fields.iter().map(|(name, _)| name.clone()).collect();
        (
            format!("\n{}\n", params.join(",\n")),
            format!("{builder_name}({})", args.join(", ")),
        )
    } else {
        let primary_input = primary_input.expect("primary input type");
        (
            render_signature_block(
                Some(format!("input: {}", primary_input.annotation)),
                &extra_params,
            ),
            render_builder_call(&builder_name, Some("input"), &extra_params),
        )
    };

    let is_void_output = output_type == "void" || output_type == "None";
    imports = dedupe_lines(imports);

    let return_type = if is_void_output {
        "None".to_string()
    } else {
        rendered_output
            .as_ref()
            .map(|output| output.annotation.clone())
            .unwrap_or_else(|| output_type.to_string())
    };

    let response_type_arg = if is_void_output {
        String::new()
    } else {
        rendered_output
            .as_ref()
            .and_then(|output| output.runtime_type.as_ref())
            .map(|runtime_type| format!(",\n        response_type={runtime_type}"))
            .unwrap_or_default()
    };

    format!(
        "{imports_block}\n\ndef {export_name}({sig_block}) -> {return_type}:\n    return get_api_client().execute(\n        {call_expr}{response_type_arg}\n    )\n",
        imports_block = imports.join("\n"),
        export_name = resolved_name.export_name,
        sig_block = signature,
        return_type = return_type,
        call_expr = call_args,
        response_type_arg = response_type_arg,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    use swagger_gen::pipeline::{
        EndpointParameter, GeneratorInput, ModelImportConfig, ProjectContext,
    };

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
            query_params: vec![],
            query_fields: vec![],
            path_params: vec![],
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

    fn resolved_py_name(file_stem: &str) -> ResolvedPyName {
        ResolvedPyName {
            file_stem: file_stem.to_string(),
            export_name: file_stem.to_string(),
            builder_name: format!("build_{file_stem}_spec"),
        }
    }

    #[test]
    fn test_renderer_id() {
        assert_eq!(PythonFunctionsRenderer.id(), "python-functions");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("getUserInfo"), "get_user_info");
        assert_eq!(
            to_snake_case("getEducationalAPIClassGetInfo"),
            "get_educational_api_class_get_info"
        );
        assert_eq!(to_snake_case("getUserID"), "get_user_id");
        assert_eq!(to_snake_case("parseHTMLString"), "parse_html_string");
    }

    #[test]
    fn test_find_common_prefix() {
        let endpoints = vec![
            make_endpoint(
                &["a"],
                "getEducationalAPIClassGetInfo",
                "GET",
                "/",
                "void",
                "void",
            ),
            make_endpoint(
                &["a"],
                "postEducationalAPIClassAdd",
                "POST",
                "/",
                "void",
                "void",
            ),
            make_endpoint(
                &["a"],
                "getEducationalAPIEnumsGetAll",
                "GET",
                "/",
                "void",
                "void",
            ),
        ];
        assert_eq!(find_common_service_prefix(&endpoints), "EducationalAPI");
    }

    #[test]
    fn test_compute_py_name_strips_prefix_and_namespace() {
        let endpoints = vec![
            make_endpoint(
                &["class"],
                "getEducationalAPIClassGetInfo",
                "GET",
                "/",
                "void",
                "void",
            ),
            make_endpoint(
                &["class"],
                "postEducationalAPIClassAdd",
                "POST",
                "/",
                "void",
                "void",
            ),
            make_endpoint(
                &["enums"],
                "getEducationalAPIEnumsGetAllAuditStatusEnum",
                "GET",
                "/",
                "void",
                "void",
            ),
            make_endpoint(
                &["class-schedule"],
                "getEducationalAPIClassScheduleGetInfo",
                "GET",
                "/",
                "void",
                "void",
            ),
        ];
        let prefix = find_common_service_prefix(&endpoints);
        assert_eq!(compute_py_name(&endpoints[0], &prefix), "get_info");
        assert_eq!(compute_py_name(&endpoints[1], &prefix), "add");
        assert_eq!(
            compute_py_name(&endpoints[2], &prefix),
            "get_all_audit_status_enum"
        );
        assert_eq!(compute_py_name(&endpoints[3], &prefix), "get_info");
    }

    #[test]
    fn test_no_prefix_with_single_endpoint() {
        let endpoints = vec![make_endpoint(&["a"], "getUser", "GET", "/", "void", "void")];
        assert_eq!(find_common_service_prefix(&endpoints), "");
    }

    #[test]
    fn test_void_input_spec() {
        let ep = make_endpoint(&["health"], "check", "GET", "/health", "void", "void");
        let content = render_spec_file(&ep, &resolved_py_name("check"), "...models");
        assert!(content.contains("def build_check_spec() -> RequestSpec:"));
        assert!(content.contains("method=\"GET\""));
        assert!(content.contains("path=\"/health\""));
    }

    #[test]
    fn test_model_input_spec() {
        let mut ep = make_endpoint(
            &["users"],
            "get_user",
            "GET",
            "/users/{id}",
            "GetUserInput",
            "User",
        );
        ep.path_fields = vec!["id".to_string()];

        let content = render_spec_file(&ep, &resolved_py_name("get_user"), "...models");
        assert!(content.contains("def build_get_user_spec("));
        assert!(content.contains("input: GetUserInput"));
        assert!(content.contains("from ...models.GetUserInput import GetUserInput"));
    }

    #[test]
    fn test_model_input_function() {
        let ep = make_endpoint(
            &["users"],
            "get_user",
            "GET",
            "/users/{id}",
            "GetUserInput",
            "User",
        );
        let content = render_function_file(
            &ep,
            &resolved_py_name("get_user"),
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
            &["users"],
            "get_user",
            "GET",
            "/users/{id}",
            "GetUserInput",
            "User",
        );

        let spec = render_spec_file(&ep, &resolved_py_name("get_user"), "...models");
        assert!(spec.contains("from ...models.GetUserInput import GetUserInput"));

        let function = render_function_file(
            &ep,
            &resolved_py_name("get_user"),
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

        let spec = render_spec_file(&ep, &resolved_py_name("upload_image"), "...models");
        assert!(spec.contains("StoreType: StoreType"));
        assert!(spec.contains("query="));

        let func = render_function_file(
            &ep,
            &resolved_py_name("upload_image"),
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
    fn test_query_only_endpoint_generates_explicit_python_parameters() {
        let mut ep = make_endpoint(
            &["user"],
            "get_login_user_wechat_un_bind",
            "GET",
            "/AuthorityAPI/User/LoginUserWechatUnBind",
            "void",
            "ResultModel",
        );
        ep.query_fields = vec!["subSystemCode".to_string()];
        ep.query_params = vec![EndpointParameter {
            name: "subSystemCode".to_string(),
            type_name: "string".to_string(),
            required: true,
        }];

        let spec = render_spec_file(
            &ep,
            &resolved_py_name("login_user_wechat_un_bind"),
            "...models",
        );
        assert!(spec.contains("def build_login_user_wechat_un_bind_spec("));
        assert!(spec.contains("subSystemCode: str"));
        assert!(spec.contains("query={ \"subSystemCode\": subSystemCode }"));

        let func = render_function_file(
            &ep,
            &resolved_py_name("login_user_wechat_un_bind"),
            "functions/user/login_user_wechat_un_bind.py",
            "...models",
        );
        assert!(func.contains("subSystemCode: str"));
        assert!(func.contains("build_login_user_wechat_un_bind_spec(subSystemCode=subSystemCode)"));
    }

    #[test]
    fn test_model_input_with_query_params_uses_explicit_kwargs() {
        let mut ep = make_endpoint(
            &["user"],
            "post_unbind",
            "POST",
            "/AuthorityAPI/User/Unbind",
            "UnbindWechatRequest",
            "ResultModel",
        );
        ep.request_body_field = Some("body".to_string());
        ep.query_fields = vec!["subSystemCode".to_string()];
        ep.query_params = vec![EndpointParameter {
            name: "subSystemCode".to_string(),
            type_name: "string".to_string(),
            required: true,
        }];

        let spec = render_spec_file(&ep, &resolved_py_name("unbind"), "...models");
        assert!(spec.contains("input: UnbindWechatRequest"));
        assert!(spec.contains("subSystemCode: str"));
        assert!(spec.contains("body=input.model_dump(by_alias=True, exclude_none=True)"));
        assert!(spec.contains("query={ \"subSystemCode\": subSystemCode }"));
        assert!(!spec.contains("input.subSystemCode"));

        let func = render_function_file(
            &ep,
            &resolved_py_name("unbind"),
            "functions/user/unbind.py",
            "...models",
        );
        assert!(func.contains("input: UnbindWechatRequest"));
        assert!(func.contains("subSystemCode: str"));
        assert!(func.contains("build_unbind_spec(input, subSystemCode=subSystemCode)"));
    }

    #[test]
    fn test_path_only_endpoint_uses_simple_namespace_for_request_input() {
        let mut ep = make_endpoint(
            &["user"],
            "get_user_detail",
            "GET",
            "/AuthorityAPI/User/{id}",
            "void",
            "User",
        );
        ep.path_fields = vec!["id".to_string()];
        ep.path_params = vec![EndpointParameter {
            name: "id".to_string(),
            type_name: "string".to_string(),
            required: true,
        }];

        let spec = render_spec_file(&ep, &resolved_py_name("get_user_detail"), "...models");
        assert!(spec.contains("from types import SimpleNamespace"));
        assert!(spec.contains("id: str"));
        assert!(spec.contains("input=SimpleNamespace(id=id)"));
    }

    #[test]
    fn test_post_with_body() {
        let mut ep = make_endpoint(
            &["users"],
            "add_user",
            "POST",
            "/users",
            "AddUserRequest",
            "User",
        );
        ep.request_body_field = Some("body".to_string());

        let spec = render_spec_file(&ep, &resolved_py_name("add_user"), "...models");
        assert!(spec.contains("body=input.model_dump(by_alias=True, exclude_none=True)"));
    }

    #[test]
    fn test_array_model_input_uses_list_annotation_and_serializes_items() {
        let mut ep = make_endpoint(
            &["answer_record"],
            "post_question_bank_api_answer_record_add",
            "POST",
            "/QuestionBankAPI/AnswerRecord/Add",
            "Array<AddAnswerRecordRequestModel>",
            "ResultModel",
        );
        ep.request_body_field = Some("body".to_string());

        let spec = render_spec_file(
            &ep,
            &resolved_py_name("post_question_bank_api_answer_record_add"),
            "...models",
        );
        assert!(spec.contains(
            "from ...models.AddAnswerRecordRequestModel import AddAnswerRecordRequestModel"
        ));
        assert!(spec.contains("input: list[AddAnswerRecordRequestModel]"));
        assert!(spec.contains(
            "body=[item_0.model_dump(by_alias=True, exclude_none=True) for item_0 in input]"
        ));
        assert!(!spec.contains("Array<AddAnswerRecordRequestModel>"));

        let func = render_function_file(
            &ep,
            &resolved_py_name("post_question_bank_api_answer_record_add"),
            "functions/answer_record/post_question_bank_api_answer_record_add.py",
            "...models",
        );
        assert!(func.contains(
            "from ...models.AddAnswerRecordRequestModel import AddAnswerRecordRequestModel"
        ));
        assert!(func.contains("from ...models.ResultModel import ResultModel"));
        assert!(func.contains("input: list[AddAnswerRecordRequestModel]"));
        assert!(func.contains(") -> ResultModel:"));
        assert!(func.contains("response_type=ResultModel"));
        assert!(!func.contains("Array<AddAnswerRecordRequestModel>"));
    }

    #[test]
    fn test_array_model_output_uses_list_annotation_without_invalid_response_type() {
        let ep = make_endpoint(
            &["users"],
            "get_users",
            "GET",
            "/users",
            "void",
            "Array<User>",
        );

        let func = render_function_file(
            &ep,
            &resolved_py_name("get_users"),
            "functions/users/get_users.py",
            "...models",
        );
        assert!(func.contains("from ...models.User import User"));
        assert!(func.contains(") -> list[User]:"));
        assert!(!func.contains("response_type=Array<User>"));
        assert!(!func.contains("response_type=list[User]"));
    }

    #[test]
    fn test_void_output_function() {
        let ep = make_endpoint(&["health"], "check", "GET", "/health", "void", "void");
        let content = render_function_file(
            &ep,
            &resolved_py_name("check"),
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
        assert_eq!(
            compute_py_name(&endpoints[0], &prefix),
            "authority_api_list"
        );
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
        assert_eq!(
            compute_fallback_py_name(&endpoints[1], &prefix),
            "user_user_add"
        );
    }

    #[test]
    fn test_resolve_final_py_names_keep_short_files_but_promote_colliding_exports() {
        let mut account_category = make_endpoint(
            &["account_category"],
            "postAuthorityAPIAccountCategoryAdd",
            "POST",
            "/AuthorityAPI/AccountCategory/Add",
            "void",
            "void",
        );
        account_category.export_name = "accountCategoryAdd".to_string();

        let mut action_authority = make_endpoint(
            &["action_authority"],
            "postAuthorityAPIActionAuthorityAdd",
            "POST",
            "/AuthorityAPI/ActionAuthority/Add",
            "void",
            "void",
        );
        action_authority.export_name = "actionAuthorityAdd".to_string();

        let resolved = resolve_final_py_names(&[account_category, action_authority]);

        assert_eq!(resolved[0].file_stem, "add");
        assert_eq!(resolved[1].file_stem, "add");
        assert_eq!(resolved[0].export_name, "account_category_add");
        assert_eq!(resolved[1].export_name, "action_authority_add");
        assert_eq!(resolved[0].builder_name, "build_account_category_add_spec");
        assert_eq!(resolved[1].builder_name, "build_action_authority_add_spec");
    }

    #[test]
    fn test_resolve_final_py_names_always_prefixes_exports_with_namespace() {
        let mut announcement = make_endpoint(
            &["announcement"],
            "postAuthorityAPIAnnouncementAdd",
            "POST",
            "/AuthorityAPI/Announcement/Add",
            "AddAnnouncementRequestModel",
            "GuidResultModel",
        );
        announcement.export_name = "announcementAdd".to_string();

        let resolved = resolve_final_py_names(&[announcement]);

        assert_eq!(resolved[0].file_stem, "add");
        assert_eq!(resolved[0].export_name, "announcement_add");
        assert_eq!(resolved[0].builder_name, "build_announcement_add_spec");
    }

    #[test]
    fn test_render_uses_short_name_inside_namespace_directory() {
        let mut action_authority = make_endpoint(
            &["action_authority"],
            "postAuthorityAPIActionAuthorityAdd",
            "POST",
            "/AuthorityAPI/ActionAuthority/Add",
            "AddActionAuthorityRequestModel",
            "GuidResultModel",
        );
        action_authority.export_name = "actionAuthorityAdd".to_string();

        let mut role = make_endpoint(
            &["role"],
            "postAuthorityAPIRoleAdd",
            "POST",
            "/AuthorityAPI/Role/Add",
            "AddRoleRequestModel",
            "GuidResultModel",
        );
        role.export_name = "roleAdd".to_string();

        let input = make_generator_input(vec![action_authority, role]);

        let output = PythonFunctionsRenderer.render(&input).unwrap();
        assert!(
            output
                .files
                .iter()
                .any(|f| f.path == "functions/action_authority/add.py")
        );
        assert!(
            output
                .files
                .iter()
                .any(|f| f.path == "spec/action_authority/add_spec.py")
        );
        assert!(
            output
                .files
                .iter()
                .any(|f| f.path == "functions/role/add.py")
        );

        let action_function = output
            .files
            .iter()
            .find(|f| f.path == "functions/action_authority/add.py")
            .unwrap();
        assert!(action_function.content.contains(
            "from ...spec.action_authority.add_spec import build_action_authority_add_spec"
        ));
        assert!(
            action_function
                .content
                .contains("def action_authority_add(")
        );
        assert!(!action_function.content.contains("async def "));
    }

    #[test]
    fn test_render_prefixes_single_namespace_function_exports() {
        let mut announcement = make_endpoint(
            &["announcement"],
            "postAuthorityAPIAnnouncementAdd",
            "POST",
            "/AuthorityAPI/Announcement/Add",
            "AddAnnouncementRequestModel",
            "GuidResultModel",
        );
        announcement.export_name = "announcementAdd".to_string();

        let output = PythonFunctionsRenderer
            .render(&make_generator_input(vec![announcement]))
            .unwrap();
        let announcement_function = output
            .files
            .iter()
            .find(|f| f.path == "functions/announcement/add.py")
            .expect("announcement function");

        assert!(
            announcement_function
                .content
                .contains("from ...spec.announcement.add_spec import build_announcement_add_spec")
        );
        assert!(
            announcement_function
                .content
                .contains("def announcement_add(")
        );
        assert!(!announcement_function.content.contains("def add("));
    }

    #[test]
    fn test_render_does_not_generate_python_init_packages() {
        let mut account_category = make_endpoint(
            &["account_category"],
            "postAuthorityAPIAccountCategoryAdd",
            "POST",
            "/AuthorityAPI/AccountCategory/Add",
            "AddAccountCategoryRequestModel",
            "GuidResultModel",
        );
        account_category.export_name = "accountCategoryAdd".to_string();

        let mut action_authority = make_endpoint(
            &["action_authority"],
            "postAuthorityAPIActionAuthorityAdd",
            "POST",
            "/AuthorityAPI/ActionAuthority/Add",
            "AddActionAuthorityRequestModel",
            "GuidResultModel",
        );
        action_authority.export_name = "actionAuthorityAdd".to_string();

        let output = PythonFunctionsRenderer
            .render(&make_generator_input(vec![
                account_category,
                action_authority,
            ]))
            .unwrap();
        assert!(
            output
                .files
                .iter()
                .all(|file| !file.path.ends_with("/__init__.py"))
        );
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
        assert!(
            output
                .files
                .iter()
                .any(|f| f.path == "functions/user/add.py")
        );
        assert!(
            output
                .files
                .iter()
                .any(|f| f.path == "functions/user/user_add.py")
        );
        assert!(output.files.iter().all(
            |f| f.path != "functions/user/add.py" || !f.content.contains("build_user_add_spec")
        ));
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
        assert!(
            output
                .files
                .iter()
                .any(|f| f.path == "functions/action_authority/add.py")
        );
        assert!(
            output
                .files
                .iter()
                .any(|f| f.path == "functions/role/add.py")
        );
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
        assert!(
            output
                .files
                .iter()
                .any(|f| f.path == "functions/user/get_login_user_info.py")
        );
        assert!(
            output
                .files
                .iter()
                .any(|f| f.path == "spec/user/get_login_user_info_spec.py")
        );
        assert!(
            output
                .files
                .iter()
                .any(|f| f.path == "functions/user/get_login_user_permissions.py")
        );
        assert!(
            !output
                .files
                .iter()
                .any(|f| f.path == "functions/user/info.py")
        );
        assert!(
            !output
                .files
                .iter()
                .any(|f| f.path == "spec/user/info_spec.py")
        );
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

        assert!(
            spec.content
                .contains("from ...models.GetUserInput import GetUserInput")
        );
        assert!(
            function
                .content
                .contains("from ...models.GetUserInput import GetUserInput")
        );
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

        assert!(
            spec.content
                .contains("from my_app.generated.models.GetUserInput import GetUserInput")
        );
        assert!(
            function
                .content
                .contains("from my_app.generated.models.GetUserInput import GetUserInput")
        );
        assert!(
            function
                .content
                .contains("from my_app.generated.models.User import User")
        );
    }
}
