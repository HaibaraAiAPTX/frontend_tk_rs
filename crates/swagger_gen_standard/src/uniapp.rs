//! UniApp renderer
//!
//! Generates TypeScript service classes for UniApp framework.

use inflector::cases::pascalcase::to_pascal_case;
use std::collections::BTreeMap;

use crate::{EndpointItem, GeneratorInput, PlannedFile, RenderOutput, Renderer};
use swagger_gen::pipeline::normalize_type_ref;

/// UniApp renderer
///
/// Generates TypeScript service classes for UniApp framework.
#[derive(Default)]
pub struct UniAppRenderer;

impl Renderer for UniAppRenderer {
    fn id(&self) -> &'static str {
        "std-uniapp"
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

/// Renders a UniApp service file for a group of endpoints.
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

/// Renders a single UniApp method.
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
        format!("{summary}  {method_name}() {{\n    return {call};\n  }}")
    } else {
        format!("{summary}  {method_name}({signature}) {{\n    return {call};\n }}")
    }
}
