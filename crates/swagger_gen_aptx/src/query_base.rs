//! Shared logic for React/Vue Query renderers
//!
//! This module contains common functionality used by both ReactQuery and VueQuery renderers.

use inflector::cases::pascalcase::to_pascal_case;

use crate::{
    get_client_call, get_client_import_lines, get_model_import_base, normalize_type_ref,
    render_type_import_block, should_use_package_import,
};

use swagger_gen::pipeline::{EndpointItem, GeneratorInput, PlannedFile, RenderOutput};

/// Query terminal type (React or Vue)
#[derive(Clone, Copy, Debug)]
pub enum QueryTerminal {
    React,
    Vue,
}

/// Renders query and mutation files for the specified terminal
pub fn render_query_terminal(
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
                content: render_query_file(
                    endpoint,
                    terminal,
                    &model_import_base,
                    use_package,
                    client_import,
                ),
            });
        }
        if endpoint.supports_mutation {
            files.push(PlannedFile {
                path: get_mutation_file_path(endpoint, terminal),
                content: render_mutation_file(
                    endpoint,
                    terminal,
                    &model_import_base,
                    use_package,
                    client_import,
                ),
            });
        }
    }

    Ok(RenderOutput {
        files,
        warnings: vec![],
    })
}

/// Returns the directory name for the terminal
pub fn terminal_dir(terminal: QueryTerminal) -> &'static str {
    match terminal {
        QueryTerminal::React => "react-query",
        QueryTerminal::Vue => "vue-query",
    }
}

/// Returns the hook factory function name for query hooks
pub fn query_hook_factory(terminal: QueryTerminal) -> &'static str {
    match terminal {
        QueryTerminal::React => "createReactQueryHooks",
        QueryTerminal::Vue => "createVueQueryHooks",
    }
}

/// Returns the hook factory function name for mutation hooks
pub fn mutation_hook_factory(terminal: QueryTerminal) -> &'static str {
    match terminal {
        QueryTerminal::React => "createReactMutationHooks",
        QueryTerminal::Vue => "createVueMutationHooks",
    }
}

/// Returns the hook alias for query hooks
pub fn query_hook_alias(terminal: QueryTerminal) -> &'static str {
    match terminal {
        QueryTerminal::React => "useAptxQuery",
        QueryTerminal::Vue => "useAptxQuery",
    }
}

/// Returns the hook alias for mutation hooks
pub fn mutation_hook_alias(terminal: QueryTerminal) -> &'static str {
    match terminal {
        QueryTerminal::React => "useAptxMutation",
        QueryTerminal::Vue => "useAptxMutation",
    }
}

/// Returns the file path for a query file
pub fn get_query_file_path(endpoint: &EndpointItem, terminal: QueryTerminal) -> String {
    let namespace = endpoint.namespace.join("/");
    format!(
        "{}/{namespace}/{}.query.ts",
        terminal_dir(terminal),
        endpoint.operation_name
    )
}

/// Returns the file path for a mutation file
pub fn get_mutation_file_path(endpoint: &EndpointItem, terminal: QueryTerminal) -> String {
    let namespace = endpoint.namespace.join("/");
    format!(
        "{}/{namespace}/{}.mutation.ts",
        terminal_dir(terminal),
        endpoint.operation_name
    )
}

/// Renders the content of a query file
pub fn render_query_file(
    endpoint: &EndpointItem,
    terminal: QueryTerminal,
    model_import_base: &str,
    use_package: bool,
    client_import: &Option<swagger_gen::pipeline::ClientImportConfig>,
) -> String {
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

/// Renders the content of a mutation file
pub fn render_mutation_file(
    endpoint: &EndpointItem,
    terminal: QueryTerminal,
    model_import_base: &str,
    use_package: bool,
    client_import: &Option<swagger_gen::pipeline::ClientImportConfig>,
) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_dir() {
        assert_eq!(terminal_dir(QueryTerminal::React), "react-query");
        assert_eq!(terminal_dir(QueryTerminal::Vue), "vue-query");
    }

    #[test]
    fn test_query_hook_factory() {
        assert_eq!(query_hook_factory(QueryTerminal::React), "createReactQueryHooks");
        assert_eq!(query_hook_factory(QueryTerminal::Vue), "createVueQueryHooks");
    }

    #[test]
    fn test_mutation_hook_factory() {
        assert_eq!(
            mutation_hook_factory(QueryTerminal::React),
            "createReactMutationHooks"
        );
        assert_eq!(
            mutation_hook_factory(QueryTerminal::Vue),
            "createVueMutationHooks"
        );
    }

    #[test]
    fn test_query_hook_alias() {
        assert_eq!(query_hook_alias(QueryTerminal::React), "useAptxQuery");
        assert_eq!(query_hook_alias(QueryTerminal::Vue), "useAptxQuery");
    }

    #[test]
    fn test_mutation_hook_alias() {
        assert_eq!(mutation_hook_alias(QueryTerminal::React), "useAptxMutation");
        assert_eq!(mutation_hook_alias(QueryTerminal::Vue), "useAptxMutation");
    }
}
