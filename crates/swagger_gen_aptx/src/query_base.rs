//! Shared logic for React/Vue Query renderers
//!
//! This module contains common functionality used by both ReactQuery and VueQuery renderers.

use crate::{
    get_client_call, get_client_import_lines, normalize_type_ref, render_type_import_block,
    resolve_file_import_path, resolve_model_import_base, should_use_package_import,
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
    current_file_path: &str,
    model_import_base: &str,
    use_package: bool,
    client_import: &Option<swagger_gen::pipeline::ClientImportConfig>,
) -> String {
    let builder = endpoint.builder_name.clone();
    let hook_name = format!(
        "use{}Query",
        inflector::cases::pascalcase::to_pascal_case(&endpoint.export_name)
    );
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
    let key_expression = if is_void_input {
        format!("{query_def}.keyPrefix")
    } else {
        format!("[...{query_def}.keyPrefix, normalizeInput(input)] as const")
    };

    let client_import_lines = get_client_import_lines(client_import);
    let client_call = get_client_call(client_import);
    let spec_file_path = format!(
        "spec/{}/{}.ts",
        endpoint.namespace.join("/"),
        endpoint.operation_name
    );
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
        "import {{ createQueryDefinition }} from \"@aptx/api-query-adapter\";\nimport type {{ QueryAdapterContext }} from \"@aptx/api-query-adapter\";\nimport {{ {hook_factory} }} from \"@aptx/{terminal_package}\";\n{client_import_lines}\nimport {{ {builder} }} from \"{spec_import_path}\";\n{type_import_block}{normalize_input_block}export const {query_def} = createQueryDefinition<{input_type}, {output_type}>({{\n  keyPrefix: [{key_prefix}] as const,\n{build_spec_line}  execute: (spec: ReturnType<typeof {builder}>, options: PerCallOptions | undefined, queryContext: QueryAdapterContext | undefined) =>\n    {client_call}.execute(spec, {{\n      ...(options ?? {{}}),\n      signal: queryContext?.signal,\n      meta: {{\n        ...(options?.meta ?? {{}}),\n        __query: queryContext?.meta,\n      }},\n    }}),\n}});\n\nexport const {key_name} = {key_signature} =>\n  {key_expression};\n\nexport const {{ {hook_alias}: {hook_name} }} = {hook_factory}({query_def});\n",
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
        key_expression = key_expression,
        spec_import_path = spec_import_path,
    )
}

/// Renders the content of a mutation file
pub fn render_mutation_file(
    endpoint: &EndpointItem,
    terminal: QueryTerminal,
    current_file_path: &str,
    model_import_base: &str,
    use_package: bool,
    client_import: &Option<swagger_gen::pipeline::ClientImportConfig>,
) -> String {
    let builder = endpoint.builder_name.clone();
    let hook_name = format!(
        "use{}Mutation",
        inflector::cases::pascalcase::to_pascal_case(&endpoint.export_name)
    );
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
    let spec_file_path = format!(
        "spec/{}/{}.ts",
        endpoint.namespace.join("/"),
        endpoint.operation_name
    );
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
        assert_eq!(
            query_hook_factory(QueryTerminal::React),
            "createReactQueryHooks"
        );
        assert_eq!(
            query_hook_factory(QueryTerminal::Vue),
            "createVueQueryHooks"
        );
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

    #[test]
    fn test_render_query_file_imports_spec_with_dynamic_relative_path() {
        let endpoint = EndpointItem {
            namespace: vec!["group".to_string(), "item".to_string()],
            operation_name: "fetchOne".to_string(),
            export_name: "itemFetchOne".to_string(),
            builder_name: "buildItemFetchOneSpec".to_string(),
            summary: None,
            method: "GET".to_string(),
            path: "/group/item".to_string(),
            input_type_name: "FetchOneInput".to_string(),
            output_type_name: "FetchOneOutput".to_string(),
            request_body_field: None,
            query_fields: vec![],
            path_fields: vec![],
            has_request_options: false,
            supports_query: true,
            supports_mutation: false,
            deprecated: false,
        };

        let content = render_query_file(
            &endpoint,
            QueryTerminal::React,
            "react-query/group/item/fetchOne.query.ts",
            "../../../spec/types",
            false,
            &None,
        );

        assert!(content.contains("from \"../../../spec/group/item/fetchOne\""));
        assert!(!content.contains(": any"));
        assert!(content.contains("QueryAdapterContext"));
        assert!(content.contains("spec: ReturnType<typeof buildItemFetchOneSpec>"));
    }

    #[test]
    fn test_render_mutation_file_should_not_have_extra_blank_lines() {
        let endpoint = EndpointItem {
            namespace: vec!["assignment".to_string()],
            operation_name: "add".to_string(),
            export_name: "assignmentAdd".to_string(),
            builder_name: "buildAssignmentAddSpec".to_string(),
            summary: None,
            method: "POST".to_string(),
            path: "/assignment/add".to_string(),
            input_type_name: "AddAssignmentRequestModel".to_string(),
            output_type_name: "GuidResultModel".to_string(),
            request_body_field: None,
            query_fields: vec![],
            path_fields: vec![],
            has_request_options: false,
            supports_query: false,
            supports_mutation: true,
            deprecated: false,
        };

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
}
