use inflector::cases::pascalcase::to_pascal_case;

use super::model::{EndpointItem, GeneratorInput, PlannedFile, RenderOutput};

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
        let mut files = Vec::new();
        for endpoint in &input.endpoints {
            files.push(PlannedFile {
                path: get_spec_file_path(endpoint),
                content: render_spec_file(endpoint),
            });
            files.push(PlannedFile {
                path: get_function_file_path(endpoint),
                content: render_function_file(endpoint),
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
    format!("api/functions/{namespace}/{}.ts", endpoint.operation_name)
}

fn render_spec_file(endpoint: &EndpointItem) -> String {
    let builder = format!("build{}Spec", to_pascal_case(&endpoint.operation_name));
    let input_type = normalize_type_ref(&endpoint.input_type_name);
    let input_import = render_type_import_line(&input_type, "../../types");
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

    format!(
        "{input_import}

export function {builder}(input: {input_type}) {{
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

fn render_function_file(endpoint: &EndpointItem) -> String {
    let builder = format!("build{}Spec", to_pascal_case(&endpoint.operation_name));
    let input_type = normalize_type_ref(&endpoint.input_type_name);
    let output_type = normalize_type_ref(&endpoint.output_type_name);
    let type_imports = render_type_import_block(
        &[input_type.as_str(), output_type.as_str()],
        "../../../spec/types",
    );
    format!(
        "import type {{ PerCallOptions }} from \"../../../client/client\";
import {{ getApiClient }} from \"../../../client/registry\";
import {{ {builder} }} from \"../../../spec/endpoints/{namespace}/{operation_name}\";
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
    let mut files = Vec::new();
    for endpoint in &input.endpoints {
        if endpoint.supports_query {
            files.push(PlannedFile {
                path: get_query_file_path(endpoint, terminal),
                content: render_query_file(endpoint, terminal),
            });
        }
        if endpoint.supports_mutation {
            files.push(PlannedFile {
                path: get_mutation_file_path(endpoint, terminal),
                content: render_mutation_file(endpoint, terminal),
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
        "api/{}/{namespace}/{}.query.ts",
        terminal_dir(terminal),
        endpoint.operation_name
    )
}

fn get_mutation_file_path(endpoint: &EndpointItem, terminal: QueryTerminal) -> String {
    let namespace = endpoint.namespace.join("/");
    format!(
        "api/{}/{namespace}/{}.mutation.ts",
        terminal_dir(terminal),
        endpoint.operation_name
    )
}

fn render_query_file(endpoint: &EndpointItem, terminal: QueryTerminal) -> String {
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
        "../../../spec/types",
    );

    format!(
        "import {{ createQueryDefinition }} from \"@aptx/api-query-adapter\";
import {{ {hook_factory} }} from \"@aptx/{terminal_package}\";
import {{ getApiClient }} from \"../../../client/registry\";
import {{ {builder} }} from \"../../../spec/endpoints/{namespace}/{operation_name}\";
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

fn render_mutation_file(endpoint: &EndpointItem, terminal: QueryTerminal) -> String {
    let builder = format!("build{}Spec", to_pascal_case(&endpoint.operation_name));
    let hook_name = format!("use{}Mutation", to_pascal_case(&endpoint.operation_name));
    let mutation_def = format!("{}MutationDef", endpoint.operation_name);
    let namespace = endpoint.namespace.join("/");

    let input_type = normalize_type_ref(&endpoint.input_type_name);
    let output_type = normalize_type_ref(&endpoint.output_type_name);
    let type_imports = render_type_import_block(
        &[input_type.as_str(), output_type.as_str()],
        "../../../spec/types",
    );

    format!(
        "import {{ createMutationDefinition }} from \"@aptx/api-query-adapter\";
import {{ {hook_factory} }} from \"@aptx/{terminal_package}\";
import {{ getApiClient }} from \"../../../client/registry\";
import {{ {builder} }} from \"../../../spec/endpoints/{namespace}/{operation_name}\";
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

fn render_type_import_line(type_name: &str, base_import_path: &str) -> String {
    if is_identifier_type(type_name) && !is_primitive_type(type_name) {
        format!("import type {{ {type_name} }} from \"{base_import_path}/{type_name}\";\n")
    } else {
        String::new()
    }
}

fn render_type_import_block(type_names: &[&str], base_import_path: &str) -> String {
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
        .map(|type_name| render_type_import_line(type_name, base_import_path))
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
