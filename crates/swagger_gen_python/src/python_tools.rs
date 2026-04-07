//! Python tools renderer for swagger_gen.
//!
//! Generates tools.json in OpenAI function calling format.

use swagger_gen::pipeline::{EndpointItem, GeneratorInput, PlannedFile, RenderOutput, Renderer};

/// Renderer that generates tools.json for OpenAI function calling.
#[derive(Default)]
pub struct PythonToolsRenderer;

impl Renderer for PythonToolsRenderer {
    fn id(&self) -> &'static str {
        "python-tools"
    }

    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String> {
        let tools: Vec<serde_json::Value> = input
            .endpoints
            .iter()
            .map(|ep| render_tool(ep))
            .collect();

        let json_output = serde_json::to_string_pretty(&tools)
            .map_err(|e| format!("Failed to serialize tools.json: {e}"))?;

        Ok(RenderOutput {
            files: vec![PlannedFile {
                path: "tools.json".to_string(),
                content: json_output,
            }],
            warnings: vec![],
        })
    }
}

fn render_tool(endpoint: &EndpointItem) -> serde_json::Value {
    let mut parameters = serde_json::json!({
        "type": "object",
        "properties": {},
    });

    for field in &endpoint.path_fields {
        parameters["properties"][field] = serde_json::json!({ "type": "string" });
    }

    for field in &endpoint.query_fields {
        parameters["properties"][field] = serde_json::json!({ "type": "string" });
    }

    if let Some(body_field) = &endpoint.request_body_field {
        parameters["properties"][body_field] = serde_json::json!({ "type": "object" });
    }

    let mut tool = serde_json::json!({
        "type": "function",
        "function": {
            "name": endpoint.export_name,
            "parameters": parameters,
        }
    });

    if let Some(summary) = &endpoint.summary {
        tool["function"]["description"] = serde_json::Value::String(summary.clone());
    }

    tool
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    fn make_endpoint(
        export_name: &str,
        method: &str,
        path: &str,
        input_type: &str,
        output_type: &str,
    ) -> EndpointItem {
        EndpointItem {
            namespace: vec!["default".to_string()],
            operation_name: export_name.to_string(),
            export_name: export_name.to_string(),
            builder_name: format!("build_{export_name}_spec"),
            summary: None,
            method: method.to_string(),
            path: path.to_string(),
            input_type_name: input_type.to_string(),
            output_type_name: output_type.to_string(),
            request_body_field: None,
            query_fields: vec![],
            path_fields: vec![],
            has_request_options: false,
            deprecated: false,
            meta: IndexMap::new(),
        }
    }

    #[test]
    fn test_renderer_id() {
        let renderer = PythonToolsRenderer;
        assert_eq!(renderer.id(), "python-tools");
    }

    #[test]
    fn test_single_endpoint_schema() {
        let mut ep = make_endpoint("usersGetUser", "GET", "/users/{id}", "GetUserInput", "User");
        ep.path_fields = vec!["id".to_string()];

        let tool = render_tool(&ep);
        assert_eq!(tool["type"], "function");
        assert_eq!(tool["function"]["name"], "usersGetUser");
        assert!(tool["function"]["parameters"]["properties"]["id"].is_object());
    }

    #[test]
    fn test_summary_as_description() {
        let mut ep = make_endpoint("getHealth", "GET", "/health", "void", "void");
        ep.summary = Some("Health check endpoint".to_string());

        let tool = render_tool(&ep);
        assert_eq!(tool["function"]["description"], "Health check endpoint");
    }

    #[test]
    fn test_query_fields_as_parameters() {
        let mut ep = make_endpoint("searchItems", "GET", "/search", "void", "Items");
        ep.query_fields = vec!["q".to_string(), "limit".to_string()];

        let tool = render_tool(&ep);
        let props = &tool["function"]["parameters"]["properties"];
        assert!(props["q"].is_object());
        assert!(props["limit"].is_object());
    }
}
