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
    get_client_call, get_client_import_lines, get_model_import_base, normalize_type_ref,
    render_type_import_block, should_use_package_import,
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
                content: render_function_file(endpoint, &model_import_base, use_package, &input.client_import),
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
    let builder = format!("build{}Spec", to_pascal_case(&endpoint.operation_name));
    let input_type = normalize_type_ref(&endpoint.input_type_name);
    let input_import = super::utils::render_type_import_line(&input_type, model_import_base, use_package);
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

fn render_function_file(endpoint: &EndpointItem, model_import_base: &str, use_package: bool, client_import: &Option<ClientImportConfig>) -> String {
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

fn render_query_terminal(
    input: &GeneratorInput,
    terminal: QueryTerminal,
) -> Result<RenderOutput, String> {
    let model_import_base = get_model_import_base(&input.model_import);
    let use_package = should_use_package_import(&input.model_import);
    let client_import = &input.client_import;
    let mut files = Vec::new();
    for endpoint in &input.endpoints {
        if endpoint.supports_query {
            files.push(PlannedFile {
                path: get_query_file_path(endpoint, terminal),
                content: render_query_file(endpoint, terminal, &model_import_base, use_package, client_import),
            });
        }
        if endpoint.supports_mutation {
            files.push(PlannedFile {
                path: get_mutation_file_path(endpoint, terminal),
                content: render_mutation_file(endpoint, terminal, &model_import_base, use_package, client_import),
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

fn render_query_file(endpoint: &EndpointItem, terminal: QueryTerminal, model_import_base: &str, use_package: bool, client_import: &Option<ClientImportConfig>) -> String {
    let builder = format!("build{}Spec", to_pascal_case(&endpoint.operation_name));
    let hook_name = format!("use{}Query", to_pascal_case(&endpoint.operation_name));
    let query_def = format!("{}QueryDef", endpoint.operation_name);
    let key_name = format!("{}Key", endpoint.operation_name);
    let namespace = endpoint.namespace.join("/");
    let key_prefix = endpoint
        .namespace
        .iter()
        .map(|item| format!("\"{item}\""))
        .chain(std::iter::once(format!("\"{}\"", endpoint.operation_name)))
        .collect::<Vec<_>>()
        .join(", ");

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
        "import {{ createQueryDefinition }} from \"@aptx/api-query-adapter\";\nimport {{ {hook_factory} }} from \"@aptx/{terminal_package}\";\n{client_import_lines}\nimport {{ {builder} }} from \"../../spec/endpoints/{namespace}/{operation_name}\";\n{type_imports}\n\nconst normalizeInput = (input: {input_type}) => JSON.stringify(input ?? null);\n\nexport const {query_def} = createQueryDefinition<{input_type}, {output_type}>({{\n  keyPrefix: [{key_prefix}] as const,\n  buildSpec: {builder},\n  execute: (spec, options: any, queryContext: any) =>\n    {client_call}.execute(spec, {{\n      ...(options ?? {{}}),\n      signal: queryContext?.signal,\n      meta: {{\n        ...(options?.meta ?? {{}}),\n        __query: queryContext?.meta,\n      }},\n    }}),\n}});\n\nexport const {key_name} = (input: {input_type}) =>\n  [...{query_def}.keyPrefix, normalizeInput(input)] as const;\n\nexport const {{ {hook_alias}: {hook_name} }} = {hook_factory}({query_def});\n",
        hook_factory = query_hook_factory(terminal),
        hook_alias = query_hook_alias(terminal),
        terminal_package = terminal_dir(terminal),
        namespace = namespace,
        operation_name = endpoint.operation_name,
        input_type = input_type,
        output_type = output_type,
        type_imports = type_imports,
        client_import_lines = client_import_lines,
        client_call = client_call,
    )
}

fn render_mutation_file(endpoint: &EndpointItem, terminal: QueryTerminal, model_import_base: &str, use_package: bool, client_import: &Option<ClientImportConfig>) -> String {
    let builder = format!("build{}Spec", to_pascal_case(&endpoint.operation_name));
    let hook_name = format!("use{}Mutation", to_pascal_case(&endpoint.operation_name));
    let mutation_def = format!("{}MutationDef", endpoint.operation_name);
    let namespace = endpoint.namespace.join("/");

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
        "import {{ createMutationDefinition }} from \"@aptx/api-query-adapter\";\nimport {{ {hook_factory} }} from \"@aptx/{terminal_package}\";\n{client_import_lines}\nimport {{ {builder} }} from \"../../spec/endpoints/{namespace}/{operation_name}\";\n{type_imports}\n\nexport const {mutation_def} = createMutationDefinition<{input_type}, {output_type}>({{\n  buildSpec: {builder},\n  execute: (spec, options) => {client_call}.execute(spec, options),\n}});\n\nexport const {{ {hook_alias}: {hook_name} }} = {hook_factory}({mutation_def});\n",
        hook_factory = mutation_hook_factory(terminal),
        hook_alias = mutation_hook_alias(terminal),
        terminal_package = terminal_dir(terminal),
        namespace = namespace,
        operation_name = endpoint.operation_name,
        input_type = input_type,
        output_type = output_type,
        type_imports = type_imports,
        client_import_lines = client_import_lines,
        client_call = client_call,
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
    let method_name = to_pascal_case(&endpoint.operation_name);
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

    let query_object = if endpoint.query_fields.is_empty() || endpoint.input_type_name == "void" {
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
        if endpoint.input_type_name == "void" {
            "undefined".to_string()
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
        format!(
            "{summary}  {method_name}() {{\n    return {call};\n  }}"
        )
    } else {
        format!(
            "{summary}  {method_name}({signature}) {{\n    return {call};\n  }}"
        )
    }
}

fn render_axios_js_function(endpoint: &EndpointItem) -> String {
    let func_name = to_pascal_case(&endpoint.operation_name);
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
        config_lines.push(format!("data: input?.{body}"));
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
    let method_name = to_pascal_case(&endpoint.operation_name);
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

    let query_object = if endpoint.query_fields.is_empty() || endpoint.input_type_name == "void" {
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
        if endpoint.input_type_name == "void" {
            "undefined".to_string()
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
        format!(
            "{summary}  {method_name}() {{\n    return {call};\n  }}"
        )
    } else {
        format!(
            "{summary}  {method_name}({signature}) {{\n    return {call};\n }}"
        )
    }
}
