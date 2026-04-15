use indexmap::IndexMap;
use inflector::cases::{
    camelcase::to_camel_case, kebabcase::to_kebab_case, pascalcase::to_pascal_case,
};
use std::collections::HashSet;
use swagger_tk::model::{OpenAPIObject, OperationObject, ParameterObjectIn, PathItemObject};

use crate::core::{ApiContext, FuncParameter};

use super::model::{EndpointItem, EndpointParameter, GeneratorInput, ProjectContext};

pub trait Parser {
    fn parse(&self, open_api: &OpenAPIObject) -> Result<GeneratorInput, String>;
}

#[derive(Default)]
pub struct OpenApiParser;

impl Parser for OpenApiParser {
    fn parse(&self, open_api: &OpenAPIObject) -> Result<GeneratorInput, String> {
        let mut endpoints = Vec::new();
        let paths = open_api
            .paths
            .as_ref()
            .ok_or_else(|| "paths not found".to_string())?;
        let mut path_keys = paths.keys().collect::<Vec<_>>();
        path_keys.sort();

        for path in path_keys {
            let path_item = paths
                .get(path)
                .ok_or_else(|| format!("can't find path data: {path}"))?;
            for (method, operation) in collect_operations(path_item) {
                let endpoint = build_endpoint(path, method, path_item, operation);
                endpoints.push(endpoint);
            }
        }
        apply_endpoint_names(&mut endpoints);

        Ok(GeneratorInput {
            project: ProjectContext {
                package_name: open_api
                    .info
                    .as_ref()
                    .map(|info| info.title.clone())
                    .unwrap_or_else(|| "generated".to_string()),
                api_base_path: None,
                terminals: vec![],
                retry_ownership: None,
            },
            endpoints,
            model_import: None,  // Will be set by configuration later
            client_import: None, // Will be set by configuration later
            output_root: None,   // Will be set by pipeline later
        })
    }
}

fn collect_operations(path_item: &PathItemObject) -> Vec<(&'static str, &OperationObject)> {
    let mut result = Vec::new();
    if let Some(op) = path_item.get.as_ref() {
        result.push(("GET", op));
    }
    if let Some(op) = path_item.post.as_ref() {
        result.push(("POST", op));
    }
    if let Some(op) = path_item.put.as_ref() {
        result.push(("PUT", op));
    }
    if let Some(op) = path_item.patch.as_ref() {
        result.push(("PATCH", op));
    }
    if let Some(op) = path_item.delete.as_ref() {
        result.push(("DELETE", op));
    }
    result
}

fn build_endpoint(
    path: &str,
    method: &str,
    path_item: &PathItemObject,
    operation: &OperationObject,
) -> EndpointItem {
    let method_lower = method.to_lowercase();
    let context = ApiContext::new(path, &method_lower, path_item, operation);
    let query_params = collect_endpoint_params(&context, ParameterObjectIn::Query);
    let path_params = collect_endpoint_params(&context, ParameterObjectIn::Path);
    let namespace = operation
        .tags
        .as_ref()
        .and_then(|tags| tags.first())
        .map(|tag| {
            tag.split('/')
                .filter(|item| !item.trim().is_empty())
                .map(to_kebab_case)
                .collect::<Vec<_>>()
        })
        .filter(|list| !list.is_empty())
        .unwrap_or_else(|| vec!["default".to_string()]);

    let input_type_name = build_input_type_name(&context);
    let operation_name = derive_operation_name(&context.func_name, method, &namespace);

    EndpointItem {
        namespace,
        operation_name,
        export_name: String::new(),
        builder_name: String::new(),
        summary: operation.summary.clone(),
        method: method.to_string(),
        path: path.to_string(),
        input_type_name,
        output_type_name: context.response_type.unwrap_or_else(|| "void".to_string()),
        request_body_field: context.request_body_name,
        query_fields: query_params.iter().map(|item| item.name.clone()).collect(),
        query_params,
        path_fields: path_params.iter().map(|item| item.name.clone()).collect(),
        path_params,
        has_request_options: true,
        deprecated: operation.deprecated.unwrap_or(false),
        meta: IndexMap::new(),
    }
}

fn collect_endpoint_params(
    context: &ApiContext,
    target: ParameterObjectIn,
) -> Vec<EndpointParameter> {
    context
        .func_parameters
        .as_ref()
        .into_iter()
        .flatten()
        .filter_map(|parameter| {
            let matches_target = matches!(
                (&parameter.r#in, &target),
                (Some(ParameterObjectIn::Query), ParameterObjectIn::Query)
                    | (Some(ParameterObjectIn::Path), ParameterObjectIn::Path)
                    | (Some(ParameterObjectIn::Header), ParameterObjectIn::Header)
                    | (Some(ParameterObjectIn::Cookie), ParameterObjectIn::Cookie)
            );

            if !matches_target {
                return None;
            }

            Some(EndpointParameter {
                name: parameter.name.clone(),
                type_name: parameter.r#type.clone(),
                required: parameter.required,
            })
        })
        .collect()
}

fn build_input_type_name(context: &ApiContext) -> String {
    if context.request_body_name.is_none() {
        return "void".to_string();
    }

    let Some(parameters) = context.func_parameters.as_ref() else {
        return "void".to_string();
    };

    // Only consider body parameters (r#in is None), exclude query/path/header/cookie params
    let body_params: Vec<_> = parameters
        .iter()
        .filter(|p| p.r#in.is_none())
        .cloned()
        .collect();

    if body_params.is_empty() {
        return "void".to_string();
    }

    if body_params.len() == 1 {
        return body_params[0].r#type.clone();
    }

    render_inline_input_type(&body_params)
}

fn render_inline_input_type(parameters: &[FuncParameter]) -> String {
    let fields = parameters
        .iter()
        .map(|parameter| {
            let field_type = if parameter.r#type.trim().is_empty() {
                "unknown".to_string()
            } else {
                parameter.r#type.clone()
            };
            format!(
                "{}{} {}",
                parameter.name,
                if parameter.required { ":" } else { "?:" },
                field_type
            )
        })
        .collect::<Vec<_>>()
        .join("; ");

    format!("{{ {fields} }}")
}

fn derive_operation_name(func_name: &str, method: &str, namespace: &[String]) -> String {
    let method_prefix = to_pascal_case(method);
    let namespace_prefix = namespace
        .iter()
        .map(|segment| to_pascal_case(segment))
        .collect::<String>();
    let full_prefix = format!("{method_prefix}MainAPI{namespace_prefix}");

    let short_name = if !namespace_prefix.is_empty() && func_name.starts_with(&full_prefix) {
        &func_name[full_prefix.len()..]
    } else {
        func_name
    };

    if short_name.trim().is_empty() {
        to_camel_case(func_name)
    } else {
        to_camel_case(short_name)
    }
}

fn apply_endpoint_names(endpoints: &mut [EndpointItem]) {
    let mut used = HashSet::<String>::new();

    for endpoint in endpoints {
        let action = extract_action_name(&endpoint.operation_name, &endpoint.namespace);
        let base = format!(
            "{}{}",
            controller_prefix(&endpoint.namespace, 1),
            to_pascal_case(&action)
        );

        let mut candidate = sanitize_reserved(&normalize_identifier(base));
        if used.contains(&candidate) {
            candidate = sanitize_reserved(&normalize_identifier(format!(
                "{}{}",
                controller_prefix(&endpoint.namespace, 2),
                to_pascal_case(&action)
            )));
        }
        if used.contains(&candidate) {
            candidate = sanitize_reserved(&normalize_identifier(format!(
                "{}{}{}",
                controller_prefix(&endpoint.namespace, 2),
                to_pascal_case(&action),
                to_pascal_case(&endpoint.method.to_lowercase())
            )));
        }
        if used.contains(&candidate) && !endpoint.path_fields.is_empty() {
            let by_suffix = endpoint
                .path_fields
                .iter()
                .map(|field| to_pascal_case(field))
                .collect::<Vec<_>>()
                .join("");
            candidate =
                sanitize_reserved(&normalize_identifier(format!("{candidate}By{by_suffix}")));
        }

        if used.contains(&candidate) {
            let mut serial = 2usize;
            let mut serial_candidate = format!("{candidate}_{serial}");
            while used.contains(&serial_candidate) {
                serial += 1;
                serial_candidate = format!("{candidate}_{serial}");
            }
            candidate = serial_candidate;
        }

        used.insert(candidate.clone());
        endpoint.export_name = candidate.clone();
        endpoint.builder_name = format!("build{}Spec", to_pascal_case(&candidate));
    }
}

fn extract_action_name(operation_name: &str, namespace: &[String]) -> String {
    let operation_words = split_identifier_words(&to_pascal_case(operation_name));
    let namespace_words = split_identifier_words(&namespace_to_pascal(namespace));

    if !namespace_words.is_empty() {
        let preferred_index = find_namespace_match_after_api(&operation_words, &namespace_words)
            .or_else(|| find_word_sequence(&operation_words, &namespace_words));

        if let Some(index) = preferred_index {
            let action_words = &operation_words[index + namespace_words.len()..];
            if !action_words.is_empty() {
                return normalize_identifier(to_camel_case(&action_words.join("")));
            }
        }
    }

    normalize_identifier(to_camel_case(operation_name))
}

fn controller_prefix(namespace: &[String], take_last: usize) -> String {
    let len = namespace.len();
    let start = len.saturating_sub(take_last);
    let raw = namespace[start..].join("-");
    let prefix = normalize_identifier(to_camel_case(&raw));
    if prefix.is_empty() {
        "default".to_string()
    } else {
        prefix
    }
}

fn namespace_to_pascal(namespace: &[String]) -> String {
    namespace
        .iter()
        .map(|segment| to_pascal_case(segment))
        .collect::<String>()
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

fn find_namespace_match_after_api(haystack: &[String], needle: &[String]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }

    haystack
        .windows(needle.len())
        .enumerate()
        .find_map(|(index, window)| {
            let matches_namespace = window
                .iter()
                .zip(needle.iter())
                .all(|(left, right)| left == right);
            let follows_api = index > 0 && haystack[index - 1].eq_ignore_ascii_case("api");

            (matches_namespace && follows_api).then_some(index)
        })
}

fn normalize_identifier(mut value: String) -> String {
    if value.trim().is_empty() {
        return "op".to_string();
    }
    if value.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        value = format!("op{}", to_pascal_case(&value));
    }
    value
}

fn sanitize_reserved(value: &str) -> String {
    const RESERVED: [&str; 12] = [
        "delete", "default", "class", "function", "new", "return", "switch", "case", "var", "let",
        "const", "import",
    ];

    if RESERVED.contains(&value) {
        format!("do{}", to_pascal_case(value))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::EndpointItem;

    fn mock_endpoint(namespace: Vec<&str>, operation_name: &str, method: &str) -> EndpointItem {
        EndpointItem {
            namespace: namespace.into_iter().map(str::to_string).collect(),
            operation_name: operation_name.to_string(),
            export_name: String::new(),
            builder_name: String::new(),
            summary: None,
            method: method.to_string(),
            path: "/demo".to_string(),
            input_type_name: "void".to_string(),
            output_type_name: "void".to_string(),
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

    #[test]
    fn apply_endpoint_names_adds_controller_prefix_and_builder() {
        let mut endpoints = vec![mock_endpoint(vec!["assignment"], "add", "POST")];
        apply_endpoint_names(&mut endpoints);
        assert_eq!(endpoints[0].export_name, "assignmentAdd");
        assert_eq!(endpoints[0].builder_name, "buildAssignmentAddSpec");
    }

    #[test]
    fn apply_endpoint_names_ensures_uniqueness() {
        let mut endpoints = vec![
            mock_endpoint(vec!["assignment"], "add", "POST"),
            mock_endpoint(vec!["main", "assignment"], "add", "POST"),
        ];
        apply_endpoint_names(&mut endpoints);
        assert_ne!(endpoints[0].export_name, endpoints[1].export_name);
    }

    #[test]
    fn apply_endpoint_names_strips_non_main_api_service_prefix() {
        let mut endpoints = vec![mock_endpoint(
            vec!["account_category"],
            "postFinancialApiAccountCategoryAdd",
            "POST",
        )];
        apply_endpoint_names(&mut endpoints);
        assert_eq!(endpoints[0].export_name, "accountCategoryAdd");
        assert_eq!(endpoints[0].builder_name, "buildAccountCategoryAddSpec");
    }

    #[test]
    fn extract_action_name_prefers_namespace_match_after_api() {
        let action = extract_action_name("postUserApiUserAdd", &["user".to_string()]);
        assert_eq!(action, "add");
    }

    #[test]
    fn sanitize_reserved_only_on_exact_keyword() {
        assert_eq!(sanitize_reserved("delete"), "doDelete");
        assert_eq!(sanitize_reserved("assignmentDelete"), "assignmentDelete");
    }
}
