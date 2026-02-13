//! Axios TypeScript renderer
//!
//! Generates TypeScript service classes using axios and tsyringe.

use inflector::cases::pascalcase::to_pascal_case;
use std::collections::BTreeMap;

use crate::{
    EndpointItem, GeneratorInput, PlannedFile, RenderOutput, Renderer,
};
use swagger_gen::pipeline::normalize_type_ref;

/// Axios TypeScript renderer
///
/// Generates TypeScript service classes with dependency injection (tsyringe)
/// and axios HTTP client.
#[derive(Default)]
pub struct AxiosTsRenderer;

impl Renderer for AxiosTsRenderer {
    fn id(&self) -> &'static str {
        "std-axios-ts"
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

/// Renders an Axios TypeScript service file for a group of endpoints.
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

/// Renders a single Axios TypeScript method.
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
            .fold(endpoint.path.clone(), |acc: String, field| {
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
