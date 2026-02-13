use inflector::cases::pascalcase::to_pascal_case;
use std::collections::BTreeMap;

use super::model::{EndpointItem, GeneratorInput, ModelImportConfig, PlannedFile, RenderOutput};

pub trait Renderer {
    fn id(&self) -> &'static str;
    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String>;
}

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
                content: render_function_file(endpoint, &model_import_base, use_package),
            });
        }
        Ok(RenderOutput {
            files,
            warnings: vec![],
        })
    }
}

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

#[derive(Default)]
pub struct AxiosTsRenderer;

#[derive(Default)]
pub struct AxiosJsRenderer;

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

fn render_function_file(endpoint: &EndpointItem, model_import_base: &str, use_package: bool) -> String {
    let builder = format!("build{}Spec", to_pascal_case(&endpoint.operation_name));
    let input_type = normalize_type_ref(&endpoint.input_type_name);
    let output_type = normalize_type_ref(&endpoint.output_type_name);
    let type_imports = render_type_import_block(
        &[input_type.as_str(), output_type.as_str()],
        model_import_base,
        use_package,
    );
    format!(
        "import type {{ PerCallOptions }} from \"../../client/client\";
import {{ getApiClient }} from \"../../client/registry\";
import {{ {builder} }} from \"../../spec/endpoints/{namespace}/{operation_name}\";
{type_imports}

export function {operation_name}(
  input: {input_type},
  options?: PerCallOptions
): Promise<{output_type}> {{
  return getApiClient().execute<{output_type}>({builder}(input), options);
}}
",
        namespace = endpoint.namespace.join("/"),
        operation_name = endpoint.operation_name,
        input_type = input_type,
        output_type = output_type,
        type_imports = type_imports,
    )
}

fn render_query_terminal(
    input: &GeneratorInput,
    terminal: QueryTerminal,
) -> Result<RenderOutput, String> {
    let model_import_base = get_model_import_base(&input.model_import);
    let use_package = should_use_package_import(&input.model_import);
    let mut files = Vec::new();
    for endpoint in &input.endpoints {
        if endpoint.supports_query {
            files.push(PlannedFile {
                path: get_query_file_path(endpoint, terminal),
                content: render_query_file(endpoint, terminal, &model_import_base, use_package),
            });
        }
        if endpoint.supports_mutation {
            files.push(PlannedFile {
                path: get_mutation_file_path(endpoint, terminal),
                content: render_mutation_file(endpoint, terminal, &model_import_base, use_package),
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

fn render_query_file(endpoint: &EndpointItem, terminal: QueryTerminal, model_import_base: &str, use_package: bool) -> String {
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

    format!(
        "import {{ createQueryDefinition }} from \"@aptx/api-query-adapter\";
import {{ {hook_factory} }} from \"@aptx/{terminal_package}\";
import {{ getApiClient }} from \"../../client/registry\";
import {{ {builder} }} from \"../../spec/endpoints/{namespace}/{operation_name}\";
{type_imports}

const normalizeInput = (input: {input_type}) => JSON.stringify(input ?? null);

export const {query_def} = createQueryDefinition<{input_type}, {output_type}>({{
  keyPrefix: [{key_prefix}] as const,
  buildSpec: {builder},
  execute: (spec, options: any, queryContext: any) =>
    getApiClient().execute(spec, {{
      ...(options ?? {{}}),
      signal: queryContext?.signal,
      meta: {{
        ...(options?.meta ?? {{}}),
        __query: queryContext?.meta,
      }},
    }}),
}});

export const {key_name} = (input: {input_type}) =>
  [...{query_def}.keyPrefix, normalizeInput(input)] as const;

export const {{ {hook_alias}: {hook_name} }} = {hook_factory}({query_def});
",
        hook_factory = query_hook_factory(terminal),
        hook_alias = query_hook_alias(terminal),
        terminal_package = terminal_dir(terminal),
        namespace = namespace,
        operation_name = endpoint.operation_name,
        input_type = input_type,
        output_type = output_type,
        type_imports = type_imports,
    )
}

fn render_mutation_file(endpoint: &EndpointItem, terminal: QueryTerminal, model_import_base: &str, use_package: bool) -> String {
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

    format!(
        "import {{ createMutationDefinition }} from \"@aptx/api-query-adapter\";
import {{ {hook_factory} }} from \"@aptx/{terminal_package}\";
import {{ getApiClient }} from \"../../client/registry\";
import {{ {builder} }} from \"../../spec/endpoints/{namespace}/{operation_name}\";
{type_imports}

export const {mutation_def} = createMutationDefinition<{input_type}, {output_type}>({{
  buildSpec: {builder},
  execute: (spec, options) => getApiClient().execute(spec, options),
}});

export const {{ {hook_alias}: {hook_name} }} = {hook_factory}({mutation_def});
",
        hook_factory = mutation_hook_factory(terminal),
        hook_alias = mutation_hook_alias(terminal),
        terminal_package = terminal_dir(terminal),
        namespace = namespace,
        operation_name = endpoint.operation_name,
        input_type = input_type,
        output_type = output_type,
        type_imports = type_imports,
    )
}

fn normalize_type_ref(type_name: &str) -> String {
    let trimmed = type_name.trim();
    if is_primitive_type(trimmed) {
        return trimmed.to_string();
    }

    if is_identifier_type(trimmed) {
        return trimmed.to_string();
    }

    "unknown".to_string()
}

/// Calculate the model import base path based on configuration
fn get_model_import_base(model_import: &Option<ModelImportConfig>) -> String {
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

/// Check if we should use package-style imports (no type file suffix)
fn should_use_package_import(model_import: &Option<ModelImportConfig>) -> bool {
    match model_import {
        None => false,
        Some(config) => config.import_type == "package",
    }
}

fn render_type_import_line(type_name: &str, base_import_path: &str, use_package: bool) -> String {
    if is_identifier_type(type_name) && !is_primitive_type(type_name) {
        if use_package {
            // Package-style import: import type { TypeName } from "package-name"
            format!("import type {{ {type_name} }} from \"{base_import_path}\";\n")
        } else {
            // Relative-style import: import type { TypeName } from "path/TypeName"
            format!("import type {{ {type_name} }} from \"{base_import_path}/{type_name}\";\n")
        }
    } else {
        String::new()
    }
}

fn render_type_import_block(type_names: &[&str], base_import_path: &str, use_package: bool) -> String {
    let mut unique = Vec::<String>::new();
    for type_name in type_names {
        if is_identifier_type(type_name)
            && !is_primitive_type(type_name)
            && !unique.iter().any(|item| item == type_name)
        {
            unique.push(type_name.to_string());
        }
    }
    unique
        .iter()
        .map(|type_name| render_type_import_line(type_name, base_import_path, use_package))
        .collect::<Vec<_>>()
        .join("")
}

fn is_identifier_type(type_name: &str) -> bool {
    let mut chars = type_name.chars();
    let first = chars.next();
    first.is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn is_primitive_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "string" | "number" | "boolean" | "void" | "object" | "unknown"
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
        "import {{ singleton }} from \"tsyringe\";
import {{ BaseService }} from \"./BaseService\";

@singleton()
export class {class_name} extends BaseService {{
{methods}
}}
"
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
            "{summary}  {method_name}() {{
    return {call};
  }}"
        )
    } else {
        format!(
            "{summary}  {method_name}({signature}) {{
    return {call};
  }}"
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
            "{summary}export function {func_name}() {{
  return axios.request({{
    {}
  }});
}}",
            config_lines.join(",\n    ")
        )
    } else {
        format!(
            "{summary}export function {func_name}({signature}) {{
  return axios.request({{
    {}
  }});
}}",
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
        "import {{ singleton }} from \"tsyringe\";
import {{ BaseService }} from \"./BaseService\";

@singleton()
export class {class_name} extends BaseService {{
{methods}
}}
"
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
            "{summary}  {method_name}() {{
    return {call};
  }}"
        )
    } else {
        format!(
            "{summary}  {method_name}({signature}) {{
    return {call};
  }}"
        )
    }
}
