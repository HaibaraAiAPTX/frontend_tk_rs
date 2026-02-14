//! Renderer trait and implementations for code generation.
//!
//! This module defines the core `Renderer` trait and provides implementations.
//!
//! **Note**: The concrete renderer implementations (FunctionsRenderer, ReactQueryRenderer, etc.)
//! will be moved to separate crates in a future migration:
//! - `swagger_gen_aptx`: @aptx-specific renderers
//! - `swagger_gen_standard`: Standard renderers

use inflector::cases::pascalcase::to_pascal_case;
use std::collections::BTreeMap;

use super::model::{ClientImportConfig, EndpointItem, GeneratorInput, PlannedFile, RenderOutput};
use super::utils::{
    get_client_call, get_client_import_lines, normalize_type_ref, render_type_import_block,
    resolve_file_import_path, resolve_model_import_base, should_use_package_import,
};

/// Core trait for code generation renderers.
///
/// All renderers (regardless of which crate they're in) must implement this trait.
pub trait Renderer: Send + Sync {
    /// Returns a unique identifier for this renderer.
    fn id(&self) -> &'static str;

    /// Renders the generated code from the input.
    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String>;
}

/// A no-operation renderer that produces no output.
#[derive(Default)]
pub struct NoopRenderer;

impl Renderer for NoopRenderer {
    fn id(&self) -> &'static str {
        "noop"
    }

    fn render(&self, _input: &GeneratorInput) -> Result<RenderOutput, String> {
        Ok(RenderOutput {
            files: vec![],
            warnings: vec![],
        })
    }
}

// ============================================================================
// @aptx-specific renderers (will be moved to swagger_gen_aptx crate)
// ============================================================================

/// Functions renderer for @aptx/api-client
#[derive(Default)]
pub struct FunctionsRenderer;

impl Renderer for FunctionsRenderer {
    fn id(&self) -> &'static str {
        "functions"
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

/// React Query renderer for @aptx/react-query
#[derive(Default)]
pub struct ReactQueryRenderer;

impl Renderer for ReactQueryRenderer {
    fn id(&self) -> &'static str {
        "react-query"
    }

    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String> {
        render_query_terminal(input, QueryTerminal::React)
    }
}

/// Vue Query renderer for @aptx/vue-query
#[derive(Default)]
pub struct VueQueryRenderer;

impl Renderer for VueQueryRenderer {
    fn id(&self) -> &'static str {
        "vue-query"
    }

    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String> {
        render_query_terminal(input, QueryTerminal::Vue)
    }
}

// ============================================================================
// Standard renderers (will be moved to swagger_gen_standard crate)
// ============================================================================

/// Axios TypeScript renderer
#[derive(Default)]
pub struct AxiosTsRenderer;

/// Axios JavaScript renderer
#[derive(Default)]
pub struct AxiosJsRenderer;

/// UniApp renderer
#[derive(Default)]
pub struct UniAppRenderer;

impl Renderer for AxiosTsRenderer {
    fn id(&self) -> &'static str {
        "axios-ts"
    }

    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String> {
        let mut grouped = BTreeMap::<String, Vec<&EndpointItem>>::new();
        for endpoint in &input.endpoints {
            let group = endpoint
                .namespace
                .first()
                .cloned()
                .unwrap_or_else(|| "default".to_string());
            grouped.entry(group).or_default().push(endpoint);
        }

        let files = grouped
            .into_iter()
            .map(|(group, endpoints)| PlannedFile {
                path: format!("{}Service.ts", to_pascal_case(&group)),
                content: render_axios_ts_service_file(&group, &endpoints),
            })
            .collect::<Vec<_>>();

        Ok(RenderOutput {
            files,
            warnings: vec![],
        })
    }
}

impl Renderer for AxiosJsRenderer {
    fn id(&self) -> &'static str {
        "axios-js"
    }

    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String> {
        let mut lines = vec!["import axios from \"axios\";".to_string(), String::new()];
        for endpoint in &input.endpoints {
            lines.push(render_axios_js_function(endpoint));
            lines.push(String::new());
        }
        Ok(RenderOutput {
            files: vec![PlannedFile {
                path: "index.js".to_string(),
                content: lines.join("\n"),
            }],
            warnings: vec![],
        })
    }
}

impl Renderer for UniAppRenderer {
    fn id(&self) -> &'static str {
        "uniapp"
    }

    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String> {
        let mut grouped = BTreeMap::<String, Vec<&EndpointItem>>::new();
        for endpoint in &input.endpoints {
            let group = endpoint
                .namespace
                .first()
                .cloned()
                .unwrap_or_else(|| "default".to_string());
            grouped.entry(group).or_default().push(endpoint);
        }

        let files = grouped
            .into_iter()
            .map(|(group, endpoints)| PlannedFile {
                path: format!("{}Service.ts", to_pascal_case(&group)),
                content: render_uniapp_service_file(&group, &endpoints),
            })
            .collect::<Vec<_>>();

        Ok(RenderOutput {
            files,
            warnings: vec![],
        })
    }
}

// ============================================================================
// Private helper functions (will be moved to respective crates)
// ============================================================================

#[derive(Clone, Copy)]
enum QueryTerminal {
    React,
    Vue,
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
    let builder = endpoint.builder_name.clone();
    let input_type = normalize_type_ref(&endpoint.input_type_name);
    let input_import =
        super::utils::render_type_import_line(&input_type, model_import_base, use_package);
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
        String::new()
    } else {
        input_import
    };

    format!(
        "{prefix}export function {builder}({signature}) {{
  return {{
    method: \"{method}\",
    path: \"{path}\",
{query_lines}{payload_field}  }};
}}
",
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
    client_import: &Option<ClientImportConfig>,
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
    format!(
        "{client_import_lines}\nimport {{ {builder} }} from \"{spec_import_path}\";\n{type_imports}\n\nexport function {operation_name}(\n{input_signature}  options?: PerCallOptions\n): Promise<{output_type}> {{\n  return {client_call}.execute<{output_type}>({builder_call}, options);\n}}\n",
        operation_name = endpoint.export_name,
        output_type = output_type,
        type_imports = type_imports,
        client_import_lines = client_import_lines,
        client_call = client_call,
        input_signature = input_signature,
        builder_call = builder_call,
        spec_import_path = spec_import_path,
    )
}

fn render_query_terminal(
    input: &GeneratorInput,
    terminal: QueryTerminal,
) -> Result<RenderOutput, String> {
    let use_package = should_use_package_import(&input.model_import);
    let client_import = &input.client_import;
    let mut files = Vec::new();
    for endpoint in &input.endpoints {
        if endpoint.supports_query {
            let query_path = get_query_file_path(endpoint, terminal);
            let query_model_import_base = resolve_model_import_base(input, &query_path);
            let query_content = render_query_file(
                endpoint,
                terminal,
                &query_path,
                &query_model_import_base,
                use_package,
                client_import,
            );
            files.push(PlannedFile {
                path: query_path,
                content: query_content,
            });
        }
        if endpoint.supports_mutation {
            let mutation_path = get_mutation_file_path(endpoint, terminal);
            let mutation_model_import_base = resolve_model_import_base(input, &mutation_path);
            let mutation_content = render_mutation_file(
                endpoint,
                terminal,
                &mutation_path,
                &mutation_model_import_base,
                use_package,
                client_import,
            );
            files.push(PlannedFile {
                path: mutation_path,
                content: mutation_content,
            });
        }
    }
    Ok(RenderOutput {
        files,
        warnings: vec![],
    })
}

fn terminal_dir(terminal: QueryTerminal) -> &'static str {
    match terminal {
        QueryTerminal::React => "react-query",
        QueryTerminal::Vue => "vue-query",
    }
}

fn query_hook_factory(terminal: QueryTerminal) -> &'static str {
    match terminal {
        QueryTerminal::React => "createReactQueryHooks",
        QueryTerminal::Vue => "createVueQueryHooks",
    }
}

fn mutation_hook_factory(terminal: QueryTerminal) -> &'static str {
    match terminal {
        QueryTerminal::React => "createReactMutationHooks",
        QueryTerminal::Vue => "createVueMutationHooks",
    }
}

fn query_hook_alias(terminal: QueryTerminal) -> &'static str {
    match terminal {
        QueryTerminal::React => "useAptxQuery",
        QueryTerminal::Vue => "useAptxQuery",
    }
}

fn mutation_hook_alias(terminal: QueryTerminal) -> &'static str {
    match terminal {
        QueryTerminal::React => "useAptxMutation",
        QueryTerminal::Vue => "useAptxMutation",
    }
}

fn get_query_file_path(endpoint: &EndpointItem, terminal: QueryTerminal) -> String {
    let namespace = endpoint.namespace.join("/");
    format!(
        "{}/{namespace}/{}.query.ts",
        terminal_dir(terminal),
        endpoint.operation_name
    )
}

fn get_mutation_file_path(endpoint: &EndpointItem, terminal: QueryTerminal) -> String {
    let namespace = endpoint.namespace.join("/");
    format!(
        "{}/{namespace}/{}.mutation.ts",
        terminal_dir(terminal),
        endpoint.operation_name
    )
}

fn render_query_file(
    endpoint: &EndpointItem,
    terminal: QueryTerminal,
    current_file_path: &str,
    model_import_base: &str,
    use_package: bool,
    client_import: &Option<ClientImportConfig>,
) -> String {
    let builder = endpoint.builder_name.clone();
    let hook_name = format!("use{}Query", to_pascal_case(&endpoint.export_name));
    let query_def = format!("{}QueryDef", endpoint.export_name);
    let key_name = format!("{}Key", endpoint.export_name);
    let key_prefix = endpoint
        .namespace
        .iter()
        .map(|item| format!("\"{item}\""))
        .chain(std::iter::once(format!("\"{}\"", endpoint.operation_name)))
        .collect::<Vec<_>>()
        .join(", ");

    let input_type = normalize_type_ref(&endpoint.input_type_name);
    let output_type = normalize_type_ref(&endpoint.output_type_name);
    let is_void_input = input_type == "void";
    let input_import_types = if is_void_input {
        vec![output_type.as_str()]
    } else {
        vec![input_type.as_str(), output_type.as_str()]
    };
    let type_imports =
        render_type_import_block(&input_import_types, model_import_base, use_package);
    let build_spec_line = if is_void_input {
        format!("  buildSpec: () => {builder}(),\n")
    } else {
        format!("  buildSpec: {builder},\n")
    };
    let normalize_input_line = if is_void_input {
        String::new()
    } else {
        format!("const normalizeInput = (input: {input_type}) => JSON.stringify(input ?? null);")
    };
    let key_signature = if is_void_input {
        "()".to_string()
    } else {
        format!("(input: {input_type})")
    };
    let key_call = if is_void_input {
        "null".to_string()
    } else {
        "normalizeInput(input)".to_string()
    };

    let client_import_lines = get_client_import_lines(client_import);
    let client_call = get_client_call(client_import);
    let spec_file_path = get_spec_file_path(endpoint);
    let spec_import_path = resolve_file_import_path(current_file_path, &spec_file_path);
    let type_import_block = if type_imports.is_empty() {
        String::new()
    } else {
        format!("{type_imports}\n")
    };
    let normalize_input_block = if normalize_input_line.is_empty() {
        String::new()
    } else {
        format!("{normalize_input_line}\n\n")
    };

    format!(
        "import {{ createQueryDefinition }} from \"@aptx/api-query-adapter\";\nimport type {{ QueryAdapterContext }} from \"@aptx/api-query-adapter\";\nimport {{ {hook_factory} }} from \"@aptx/{terminal_package}\";\n{client_import_lines}\nimport {{ {builder} }} from \"{spec_import_path}\";\n{type_import_block}{normalize_input_block}export const {query_def} = createQueryDefinition<{input_type}, {output_type}>({{\n  keyPrefix: [{key_prefix}] as const,\n{build_spec_line}  execute: (spec: ReturnType<typeof {builder}>, options: PerCallOptions | undefined, queryContext: QueryAdapterContext | undefined) =>\n    {client_call}.execute(spec, {{\n      ...(options ?? {{}}),\n      signal: queryContext?.signal,\n      meta: {{\n        ...(options?.meta ?? {{}}),\n        __query: queryContext?.meta,\n      }},\n    }}),\n}});\n\nexport const {key_name} = {key_signature} =>\n  [...{query_def}.keyPrefix, {key_call}] as const;\n\nexport const {{ {hook_alias}: {hook_name} }} = {hook_factory}({query_def});\n",
        hook_factory = query_hook_factory(terminal),
        hook_alias = query_hook_alias(terminal),
        terminal_package = terminal_dir(terminal),
        input_type = input_type,
        output_type = output_type,
        client_import_lines = client_import_lines,
        client_call = client_call,
        type_import_block = type_import_block,
        normalize_input_block = normalize_input_block,
        build_spec_line = build_spec_line,
        key_signature = key_signature,
        key_call = key_call,
        spec_import_path = spec_import_path,
    )
}

fn render_mutation_file(
    endpoint: &EndpointItem,
    terminal: QueryTerminal,
    current_file_path: &str,
    model_import_base: &str,
    use_package: bool,
    client_import: &Option<ClientImportConfig>,
) -> String {
    let builder = endpoint.builder_name.clone();
    let hook_name = format!("use{}Mutation", to_pascal_case(&endpoint.export_name));
    let mutation_def = format!("{}MutationDef", endpoint.export_name);

    let input_type = normalize_type_ref(&endpoint.input_type_name);
    let output_type = normalize_type_ref(&endpoint.output_type_name);
    let is_void_input = input_type == "void";
    let input_import_types = if is_void_input {
        vec![output_type.as_str()]
    } else {
        vec![input_type.as_str(), output_type.as_str()]
    };
    let type_imports =
        render_type_import_block(&input_import_types, model_import_base, use_package);
    let build_spec_line = if is_void_input {
        format!("  buildSpec: () => {builder}(),\n")
    } else {
        format!("  buildSpec: {builder},\n")
    };

    let client_import_lines = get_client_import_lines(client_import);
    let client_call = get_client_call(client_import);
    let spec_file_path = get_spec_file_path(endpoint);
    let spec_import_path = resolve_file_import_path(current_file_path, &spec_file_path);
    let type_import_block = if type_imports.is_empty() {
        String::new()
    } else {
        format!("{type_imports}\n")
    };

    format!(
        "import {{ createMutationDefinition }} from \"@aptx/api-query-adapter\";\nimport {{ {hook_factory} }} from \"@aptx/{terminal_package}\";\n{client_import_lines}\nimport {{ {builder} }} from \"{spec_import_path}\";\n{type_import_block}export const {mutation_def} = createMutationDefinition<{input_type}, {output_type}>({{\n{build_spec_line}  execute: (spec: ReturnType<typeof {builder}>, options?: PerCallOptions) => {client_call}.execute(spec, options),\n}});\n\nexport const {{ {hook_alias}: {hook_name} }} = {hook_factory}({mutation_def});\n",
        hook_factory = mutation_hook_factory(terminal),
        hook_alias = mutation_hook_alias(terminal),
        terminal_package = terminal_dir(terminal),
        input_type = input_type,
        output_type = output_type,
        client_import_lines = client_import_lines,
        client_call = client_call,
        build_spec_line = build_spec_line,
        spec_import_path = spec_import_path,
        type_import_block = type_import_block,
    )
}

fn render_axios_ts_service_file(group: &str, endpoints: &[&EndpointItem]) -> String {
    let class_name = format!("{}Service", to_pascal_case(group));
    let methods = endpoints
        .iter()
        .map(|endpoint| render_axios_ts_method(endpoint))
        .collect::<Vec<_>>()
        .join("\n\n");

    format!(
        "import {{ singleton }} from \"tsyringe\";\nimport {{ BaseService }} from \"./BaseService\";\n\n@singleton()\nexport class {class_name} extends BaseService {{\n{methods}\n}}\n"
    )
}

fn render_axios_ts_method(endpoint: &EndpointItem) -> String {
    let method_name = to_pascal_case(&endpoint.export_name);
    let method = endpoint.method.to_lowercase();
    let output_type = normalize_type_ref(&endpoint.output_type_name);
    let signature = if endpoint.input_type_name == "void" {
        String::new()
    } else {
        format!("input: {}", normalize_type_ref(&endpoint.input_type_name))
    };

    let summary = endpoint
        .summary
        .as_ref()
        .map(|text| format!("  /** {} */\n", text))
        .unwrap_or_default();

    let url = if endpoint.path_fields.is_empty() {
        format!("\"{}\"", endpoint.path)
    } else {
        let path = endpoint
            .path_fields
            .iter()
            .fold(endpoint.path.clone(), |acc, field| {
                acc.replace(&format!("{{{field}}}"), &format!("${{input.{field}}}"))
            });
        format!("`{path}`")
    };

    let input_type = normalize_type_ref(&endpoint.input_type_name);
    let query_object = if endpoint.query_fields.is_empty() || input_type == "void" {
        None
    } else {
        Some(format!(
            "{{ {} }}",
            endpoint
                .query_fields
                .iter()
                .map(|field| format!("{field}: input.{field}"))
                .collect::<Vec<_>>()
                .join(", ")
        ))
    };

    let body_value = endpoint.request_body_field.as_ref().map(|field| {
        if input_type == "void" {
            "undefined".to_string()
        } else if endpoint.query_fields.is_empty() && endpoint.path_fields.is_empty() {
            "input".to_string()
        } else {
            format!("input.{field}")
        }
    });

    let call = if ["post", "put", "patch"].contains(&method.as_str()) {
        match (body_value, query_object) {
            (None, None) => format!("this.{method}<{output_type}>({url})"),
            (Some(body), None) => format!("this.{method}<{output_type}>({url}, {body})"),
            (None, Some(query)) => {
                format!("this.{method}<{output_type}>({url}, null, {{ params: {query} }})")
            }
            (Some(body), Some(query)) => {
                format!("this.{method}<{output_type}>({url}, {body}, {{ params: {query} }})")
            }
        }
    } else {
        match query_object {
            None => format!("this.{method}<{output_type}>({url})"),
            Some(query) => format!("this.{method}<{output_type}>({url}, {{ params: {query} }})"),
        }
    };

    if signature.is_empty() {
        format!("{summary}  {method_name}() {{\n    return {call};\n  }}")
    } else {
        format!("{summary}  {method_name}({signature}) {{\n    return {call};\n  }}")
    }
}

fn render_axios_js_function(endpoint: &EndpointItem) -> String {
    let func_name = to_pascal_case(&endpoint.export_name);
    let method = endpoint.method.to_lowercase();
    let signature = if endpoint.input_type_name == "void" {
        String::new()
    } else {
        "input".to_string()
    };

    let summary = endpoint
        .summary
        .as_ref()
        .map(|text| format!("/** {} */\n", text))
        .unwrap_or_default();

    let url = if endpoint.path_fields.is_empty() {
        format!("\"{}\"", endpoint.path)
    } else {
        let path = endpoint
            .path_fields
            .iter()
            .fold(endpoint.path.clone(), |acc, field| {
                acc.replace(&format!("{{{field}}}"), &format!("${{input?.{field}}}"))
            });
        format!("`{path}`")
    };

    let mut config_lines = vec![format!("url: {url}"), format!("method: \"{method}\"")];
    if let Some(body) = endpoint.request_body_field.as_ref() {
        if endpoint.query_fields.is_empty() && endpoint.path_fields.is_empty() {
            config_lines.push("data: input".to_string());
        } else {
            config_lines.push(format!("data: input?.{body}"));
        }
    }
    if !endpoint.query_fields.is_empty() {
        let query = endpoint
            .query_fields
            .iter()
            .map(|field| format!("{field}: input?.{field}"))
            .collect::<Vec<_>>()
            .join(", ");
        config_lines.push(format!("params: {{ {query} }}"));
    }

    if signature.is_empty() {
        format!(
            "{summary}export function {func_name}() {{\n  return axios.request({{\n    {}\n  }});\n}}",
            config_lines.join(",\n    ")
        )
    } else {
        format!(
            "{summary}export function {func_name}({signature}) {{\n  return axios.request({{\n    {}\n  }});\n}}",
            config_lines.join(",\n    ")
        )
    }
}

fn render_uniapp_service_file(group: &str, endpoints: &[&EndpointItem]) -> String {
    let class_name = format!("{}Service", to_pascal_case(group));
    let methods = endpoints
        .iter()
        .map(|endpoint| render_uniapp_method(endpoint))
        .collect::<Vec<_>>()
        .join("\n\n");

    format!(
        "import {{ singleton }} from \"tsyringe\";\nimport {{ BaseService }} from \"./BaseService\";\n\n@singleton()\nexport class {class_name} extends BaseService {{\n{methods}\n}}\n"
    )
}

fn render_uniapp_method(endpoint: &EndpointItem) -> String {
    let method_name = to_pascal_case(&endpoint.export_name);
    let method = endpoint.method.to_lowercase();
    let output_type = normalize_type_ref(&endpoint.output_type_name);
    let signature = if endpoint.input_type_name == "void" {
        String::new()
    } else {
        format!("input: {}", normalize_type_ref(&endpoint.input_type_name))
    };

    let summary = endpoint
        .summary
        .as_ref()
        .map(|text| format!("  /** {} */\n", text))
        .unwrap_or_default();

    let url = if endpoint.path_fields.is_empty() {
        format!("\"{}\"", endpoint.path)
    } else {
        let path = endpoint
            .path_fields
            .iter()
            .fold(endpoint.path.clone(), |acc, field| {
                acc.replace(&format!("{{{field}}}"), &format!("${{input.{field}}}"))
            });
        format!("`{path}`")
    };

    let input_type = normalize_type_ref(&endpoint.input_type_name);
    let query_object = if endpoint.query_fields.is_empty() || input_type == "void" {
        None
    } else {
        Some(format!(
            "{{ {} }}",
            endpoint
                .query_fields
                .iter()
                .map(|field| format!("{field}: input.{field}"))
                .collect::<Vec<_>>()
                .join(", ")
        ))
    };

    let body_value = endpoint.request_body_field.as_ref().map(|field| {
        if input_type == "void" {
            "undefined".to_string()
        } else if endpoint.query_fields.is_empty() && endpoint.path_fields.is_empty() {
            "input".to_string()
        } else {
            format!("input.{field}")
        }
    });

    let call = match (body_value, query_object) {
        (None, None) => format!("this.{method}<{output_type}>({url})"),
        (Some(body), None) => format!("this.{method}<{output_type}>({url}, {body})"),
        (None, Some(query)) => {
            format!("this.{method}<{output_type}>({url}, {{ params: {query} }})")
        }
        (Some(body), Some(query)) => {
            format!("this.{method}<{output_type}>({url}, {body}, {{ params: {query} }})")
        }
    };

    if signature.is_empty() {
        format!("{summary}  {method_name}() {{\n    return {call};\n  }}")
    } else {
        format!("{summary}  {method_name}({signature}) {{\n    return {call};\n }}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_endpoint(namespace: Vec<&str>, operation_name: &str) -> EndpointItem {
        EndpointItem {
            namespace: namespace.into_iter().map(str::to_string).collect(),
            operation_name: operation_name.to_string(),
            export_name: operation_name.to_string(),
            builder_name: format!("build{}Spec", to_pascal_case(operation_name)),
            summary: None,
            method: "POST".to_string(),
            path: "/demo".to_string(),
            input_type_name: "DemoInput".to_string(),
            output_type_name: "DemoOutput".to_string(),
            request_body_field: None,
            query_fields: vec![],
            path_fields: vec![],
            has_request_options: false,
            supports_query: true,
            supports_mutation: true,
            deprecated: false,
        }
    }

    #[test]
    fn function_file_should_import_spec_with_dynamic_relative_path() {
        let endpoint = make_endpoint(vec!["assignment"], "add");
        let content = render_function_file(
            &endpoint,
            "functions/api/assignment/add.ts",
            "../../../spec/types",
            false,
            &None,
        );

        assert!(content.contains("from \"../../../spec/endpoints/assignment/add\""));
    }

    #[test]
    fn query_file_should_import_spec_with_dynamic_relative_path() {
        let endpoint = make_endpoint(vec!["group", "item"], "queryOne");
        let content = render_query_file(
            &endpoint,
            QueryTerminal::React,
            "react-query/group/item/queryOne.query.ts",
            "../../../spec/types",
            false,
            &None,
        );

        assert!(content.contains("from \"../../../spec/endpoints/group/item/queryOne\""));
        assert!(!content.contains(": any"));
        assert!(content.contains("QueryAdapterContext"));
        assert!(content.contains("spec: ReturnType<typeof buildQueryOneSpec>"));
    }

    #[test]
    fn query_file_void_input_should_not_emit_empty_normalize_input_function() {
        let mut endpoint = make_endpoint(vec!["application"], "ping");
        endpoint.input_type_name = "void".to_string();
        endpoint.builder_name = "buildApplicationPingSpec".to_string();
        let content = render_query_file(
            &endpoint,
            QueryTerminal::React,
            "react-query/application/ping.query.ts",
            "../../../spec/types",
            false,
            &None,
        );

        assert!(!content.contains("const normalizeInput = () => \"null\";"));
        assert!(content.contains("[...pingQueryDef.keyPrefix, null]"));
        assert!(!content.contains("\n\n\n\n"));
    }

    #[test]
    fn mutation_file_should_not_have_extra_blank_lines() {
        let endpoint = make_endpoint(vec!["assignment"], "add");
        let content = render_mutation_file(
            &endpoint,
            QueryTerminal::React,
            "react-query/assignment/add.mutation.ts",
            "../../domains",
            false,
            &None,
        );

        assert!(!content.contains("\n\n\n"));
    }

    #[test]
    fn spec_file_should_use_typed_fields_without_any_and_aligned_indent() {
        let mut endpoint = make_endpoint(vec!["application"], "setClockInReward");
        endpoint.input_type_name = "{ reward?: number }".to_string();
        endpoint.method = "PUT".to_string();
        endpoint.query_fields = vec!["reward".to_string()];

        let content = render_spec_file(&endpoint, "../../../spec/types", false);
        assert!(content.contains("    query: { reward: input.reward },"));
        assert!(!content.contains("as any"));
    }

    #[test]
    fn spec_file_should_import_nested_model_types_from_inline_input_type() {
        let mut endpoint = make_endpoint(vec!["stored-file"], "uploadImage");
        endpoint.builder_name = "buildStoredFileUploadImageSpec".to_string();
        endpoint.input_type_name = "{ StoreType: StoreType; body?: object }".to_string();
        endpoint.query_fields = vec!["StoreType".to_string()];
        let content = render_spec_file(&endpoint, "../../../domains", false);

        assert!(content.contains("import type { StoreType } from \"../../../domains/StoreType\";"));
    }
}
