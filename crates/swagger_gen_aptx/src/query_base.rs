//! Shared logic for React/Vue Query renderers
//!
//! This module contains common functionality used by both ReactQuery and VueQuery renderers.

use crate::META_SUPPORTS_QUERY;
use crate::{
    ResolvedTsName, get_client_call, get_client_import_lines, normalize_type_ref,
    render_type_import_block, resolve_file_import_path, resolve_final_ts_names,
    resolve_model_import_base, should_use_package_import,
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
    let resolved_names = resolve_final_ts_names(&input.endpoints);

    for (endpoint, resolved_name) in input.endpoints.iter().zip(resolved_names.iter()) {
        let supports_query = endpoint.meta.get(META_SUPPORTS_QUERY) == Some(&"true".to_string());

        if supports_query {
            let query_path = get_query_file_path(endpoint, resolved_name, terminal);
            let query_model_import_base = resolve_model_import_base(input, &query_path);
            let query_content = render_query_file(
                endpoint,
                resolved_name,
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
        } else {
            // If not a query, it's a mutation
            let mutation_path = get_mutation_file_path(endpoint, resolved_name, terminal);
            let mutation_model_import_base = resolve_model_import_base(input, &mutation_path);
            let mutation_content = render_mutation_file(
                endpoint,
                resolved_name,
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

/// Returns the directory name for the terminal (used for file paths)
pub fn terminal_dir(terminal: QueryTerminal) -> &'static str {
    match terminal {
        QueryTerminal::React => "react-query",
        QueryTerminal::Vue => "vue-query",
    }
}

/// Returns the package name for the terminal (used for imports)
pub fn terminal_package_name(terminal: QueryTerminal) -> &'static str {
    match terminal {
        QueryTerminal::React => "api-query-react",
        QueryTerminal::Vue => "api-query-vue",
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
pub fn get_query_file_path(
    endpoint: &EndpointItem,
    resolved_name: &ResolvedTsName,
    terminal: QueryTerminal,
) -> String {
    let namespace = endpoint.namespace.join("/");
    format!(
        "{}/{namespace}/{}.query.ts",
        terminal_dir(terminal),
        resolved_name.file_stem
    )
}

/// Returns the file path for a mutation file
pub fn get_mutation_file_path(
    endpoint: &EndpointItem,
    resolved_name: &ResolvedTsName,
    terminal: QueryTerminal,
) -> String {
    let namespace = endpoint.namespace.join("/");
    format!(
        "{}/{namespace}/{}.mutation.ts",
        terminal_dir(terminal),
        resolved_name.file_stem
    )
}

/// Renders the content of a query file
pub fn render_query_file(
    endpoint: &EndpointItem,
    resolved_name: &ResolvedTsName,
    terminal: QueryTerminal,
    current_file_path: &str,
    model_import_base: &str,
    use_package: bool,
    client_import: &Option<swagger_gen::pipeline::ClientImportConfig>,
) -> String {
    let builder = resolved_name.builder_name.clone();
    let hook_name = format!(
        "use{}Query",
        inflector::cases::pascalcase::to_pascal_case(&resolved_name.export_name)
    );
    let query_def = format!("{}QueryDef", resolved_name.export_name);
    let key_name = format!("{}Key", resolved_name.export_name);
    let key_prefix = endpoint
        .namespace
        .iter()
        .map(|item| format!("\"{item}\""))
        .chain(std::iter::once(format!("\"{}\"", resolved_name.file_stem)))
        .collect::<Vec<_>>()
        .join(", ");
    let key_prefix_array = format!("[{key_prefix}] as const");

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
        resolved_name.file_stem
    );
    let spec_import_path = resolve_file_import_path(current_file_path, &spec_file_path);
    let type_import_block = if type_imports.is_empty() {
        String::new()
    } else {
        format!("{type_imports}\n")
    };

    format!(
        "import {{ createQueryDefinition }} from \"@aptx/api-query-adapter\";\nimport {{ {hook_factory} }} from \"@aptx/{terminal_package}\";\n{client_import_lines}\nimport {{ {builder} }} from \"{spec_import_path}\";\n{type_import_block}const {query_def} = createQueryDefinition<{input_type}, {output_type}>({{\n  keyPrefix: {key_prefix_array},\n{build_spec_line}  execute: (spec: ReturnType<typeof {builder}>, options?: PerCallOptions) =>\n    {client_call}.execute(spec, options),\n}});\n\nexport const {key_name} = {query_def}.key;\n\nexport const {{ {hook_alias}: {hook_name} }} = {hook_factory}({query_def});\n",
        hook_factory = query_hook_factory(terminal),
        hook_alias = query_hook_alias(terminal),
        terminal_package = terminal_package_name(terminal),
        input_type = input_type,
        output_type = output_type,
        client_import_lines = client_import_lines,
        client_call = client_call,
        type_import_block = type_import_block,
        build_spec_line = build_spec_line,
        key_prefix_array = key_prefix_array,
        spec_import_path = spec_import_path,
    )
}

/// Renders the content of a mutation file
pub fn render_mutation_file(
    endpoint: &EndpointItem,
    resolved_name: &ResolvedTsName,
    terminal: QueryTerminal,
    current_file_path: &str,
    model_import_base: &str,
    use_package: bool,
    client_import: &Option<swagger_gen::pipeline::ClientImportConfig>,
) -> String {
    let builder = resolved_name.builder_name.clone();
    let hook_name = format!(
        "use{}Mutation",
        inflector::cases::pascalcase::to_pascal_case(&resolved_name.export_name)
    );
    let mutation_def = format!("{}MutationDef", resolved_name.export_name);

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
        resolved_name.file_stem
    );
    let spec_import_path = resolve_file_import_path(current_file_path, &spec_file_path);
    let type_import_block = if type_imports.is_empty() {
        String::new()
    } else {
        format!("{type_imports}\n")
    };

    format!(
        "import {{ createMutationDefinition }} from \"@aptx/api-query-adapter\";\nimport {{ {hook_factory} }} from \"@aptx/{terminal_package}\";\n{client_import_lines}\nimport {{ {builder} }} from \"{spec_import_path}\";\n{type_import_block}const {mutation_def} = createMutationDefinition<{input_type}, {output_type}>({{\n{build_spec_line}  execute: (spec: ReturnType<typeof {builder}>, options?: PerCallOptions) => {client_call}.execute(spec, options),\n}});\n\nexport const {{ {hook_alias}: {hook_name} }} = {hook_factory}({mutation_def});\n",
        hook_factory = mutation_hook_factory(terminal),
        hook_alias = mutation_hook_alias(terminal),
        terminal_package = terminal_package_name(terminal),
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
    use indexmap::IndexMap;
    use swagger_gen::pipeline::{GeneratorInput, ProjectContext};

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

    #[test]
    fn test_terminal_dir() {
        assert_eq!(terminal_dir(QueryTerminal::React), "react-query");
        assert_eq!(terminal_dir(QueryTerminal::Vue), "vue-query");
    }

    #[test]
    fn test_terminal_package_name() {
        assert_eq!(
            terminal_package_name(QueryTerminal::React),
            "api-query-react"
        );
        assert_eq!(terminal_package_name(QueryTerminal::Vue), "api-query-vue");
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
        let mut meta = IndexMap::new();
        meta.insert(META_SUPPORTS_QUERY.to_string(), "true".to_string());

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
            query_params: vec![],
            query_fields: vec![],
            path_params: vec![],
            path_fields: vec![],
            has_request_options: false,
            deprecated: false,
            meta,
        };

        let content = render_query_file(
            &endpoint,
            &ResolvedTsName {
                file_stem: "fetchOne".to_string(),
                export_name: "fetchOne".to_string(),
                builder_name: "buildFetchOneSpec".to_string(),
            },
            QueryTerminal::React,
            "react-query/group/item/fetchOne.query.ts",
            "../../../spec/types",
            false,
            &None,
        );

        assert!(content.contains("from \"../../../spec/group/item/fetchOne\""));
        assert!(!content.contains(": any"));
        assert!(content.contains("spec: ReturnType<typeof buildFetchOneSpec>"));
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
            query_params: vec![],
            query_fields: vec![],
            path_params: vec![],
            path_fields: vec![],
            has_request_options: false,
            deprecated: false,
            meta: IndexMap::new(),
        };

        let content = render_mutation_file(
            &endpoint,
            &ResolvedTsName {
                file_stem: "add".to_string(),
                export_name: "add".to_string(),
                builder_name: "buildAddSpec".to_string(),
            },
            QueryTerminal::React,
            "react-query/assignment/add.mutation.ts",
            "../../domains",
            false,
            &None,
        );

        assert!(!content.contains("\n\n\n"));
    }

    #[test]
    fn test_render_query_terminal_uses_namespace_prefixed_query_names() {
        let mut meta = IndexMap::new();
        meta.insert(META_SUPPORTS_QUERY.to_string(), "true".to_string());

        let input = make_generator_input(vec![EndpointItem {
            namespace: vec!["user".to_string()],
            operation_name: "getAuthorityAPIUserGetLoginUserInfo".to_string(),
            export_name: "userGetLoginUserInfo".to_string(),
            builder_name: "buildUserGetLoginUserInfoSpec".to_string(),
            summary: None,
            method: "GET".to_string(),
            path: "/AuthorityAPI/User/GetLoginUserInfo".to_string(),
            input_type_name: "void".to_string(),
            output_type_name: "LoginUserInfo".to_string(),
            request_body_field: None,
            query_params: vec![],
            query_fields: vec![],
            path_params: vec![],
            path_fields: vec![],
            has_request_options: false,
            deprecated: false,
            meta,
        }]);

        let output = render_query_terminal(&input, QueryTerminal::React).unwrap();
        let file = output
            .files
            .iter()
            .find(|f| f.path == "react-query/user/getLoginUserInfo.query.ts")
            .expect("query file");

        assert!(
            file.content
                .contains("import { buildUserGetLoginUserInfoSpec }")
        );
        assert!(file.content.contains("const userGetLoginUserInfoQueryDef"));
        assert!(
            file.content
                .contains("export const userGetLoginUserInfoKey")
        );
        assert!(file.content.contains("useUserGetLoginUserInfoQuery"));
        assert!(!file.content.contains("useInfoQuery"));
    }

    #[test]
    fn test_render_query_terminal_uses_namespace_prefixed_mutation_names() {
        let input = make_generator_input(vec![EndpointItem {
            namespace: vec!["action_authority".to_string()],
            operation_name: "postAuthorityAPIActionAuthorityAdd".to_string(),
            export_name: "actionAuthorityAdd".to_string(),
            builder_name: "buildActionAuthorityAddSpec".to_string(),
            summary: None,
            method: "POST".to_string(),
            path: "/AuthorityAPI/ActionAuthority/Add".to_string(),
            input_type_name: "AddActionAuthorityRequestModel".to_string(),
            output_type_name: "GuidResultModel".to_string(),
            request_body_field: None,
            query_params: vec![],
            query_fields: vec![],
            path_params: vec![],
            path_fields: vec![],
            has_request_options: false,
            deprecated: false,
            meta: IndexMap::new(),
        }]);

        let output = render_query_terminal(&input, QueryTerminal::React).unwrap();
        let file = output
            .files
            .iter()
            .find(|f| f.path == "react-query/action_authority/add.mutation.ts")
            .expect("mutation file");

        assert!(
            file.content
                .contains("import { buildActionAuthorityAddSpec }")
        );
        assert!(file.content.contains("const actionAuthorityAddMutationDef"));
        assert!(file.content.contains("useActionAuthorityAddMutation"));
    }
}
