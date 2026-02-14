use inflector::cases::{
    camelcase::to_camel_case, kebabcase::to_kebab_case, pascalcase::to_pascal_case,
};
use std::collections::HashSet;
use swagger_tk::model::{OpenAPIObject, OperationObject, PathItemObject};

use crate::core::{ApiContext, FuncParameter};

use super::model::{EndpointItem, GeneratorInput, ProjectContext};

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
        query_fields: context
            .query_params_list
            .unwrap_or_default()
            .into_iter()
            .map(|item| item.name)
            .collect(),
        path_fields: context
            .path_params_list
            .unwrap_or_default()
            .into_iter()
            .map(|item| item.name)
            .collect(),
        has_request_options: true,
        supports_query: method == "GET",
        supports_mutation: method != "GET",
        deprecated: operation.deprecated.unwrap_or(false),
    }
}

fn build_input_type_name(context: &ApiContext) -> String {
    let Some(parameters) = context.func_parameters.as_ref() else {
        return "void".to_string();
    };

    if parameters.len() == 1 {
        let single = &parameters[0];
        if context.request_body_name.is_some() && single.r#in.is_none() {
            return single.r#type.clone();
        }
        return render_inline_input_type(parameters);
    }

    render_inline_input_type(parameters)
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
        let action = normalize_identifier(to_camel_case(&endpoint.operation_name));
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
            query_fields: vec![],
            path_fields: vec![],
            has_request_options: false,
            supports_query: false,
            supports_mutation: true,
            deprecated: false,
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
    fn sanitize_reserved_only_on_exact_keyword() {
        assert_eq!(sanitize_reserved("delete"), "doDelete");
        assert_eq!(sanitize_reserved("assignmentDelete"), "assignmentDelete");
    }
}
