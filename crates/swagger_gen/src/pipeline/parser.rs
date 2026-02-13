use inflector::cases::{camelcase::to_camel_case, kebabcase::to_kebab_case};
use swagger_tk::model::{OpenAPIObject, OperationObject, PathItemObject};

use crate::core::ApiContext;

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
            model_import: None, // Will be set by configuration later
            client_import: None, // Will be set by configuration later
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

    let input_type_name = match context.func_parameters.as_ref() {
        None => "void".to_string(),
        Some(parameters) if parameters.len() == 1 => parameters[0].r#type.clone(),
        Some(_) => format!("{}Input", context.func_name),
    };

    EndpointItem {
        namespace,
        operation_name: to_camel_case(&context.func_name),
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
