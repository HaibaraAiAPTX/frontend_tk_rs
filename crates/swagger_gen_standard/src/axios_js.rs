//! Axios JavaScript renderer
//!
//! Generates JavaScript functions using axios.

use inflector::cases::pascalcase::to_pascal_case;

use crate::{EndpointItem, GeneratorInput, PlannedFile, RenderOutput, Renderer};

/// Axios JavaScript renderer
///
/// Generates JavaScript functions using axios.
#[derive(Default)]
pub struct AxiosJsRenderer;

impl Renderer for AxiosJsRenderer {
    fn id(&self) -> &'static str {
        "std-axios-js"
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

/// Renders a single Axios JavaScript function.
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
            .fold(endpoint.path.clone(), |acc: String, field| {
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
            "{summary}export function {func_name}() {{\n  return axios.request({{\n    {}\n  }});\n}}",
            config_lines.join(",\n    ")
        )
    } else {
        format!(
            "{summary}export function {func_name}({signature}) {{\n  return axios.request({{\n    {}\n  }});\n}}",
            config_lines.join(",\n    ")
        )
    }
}
