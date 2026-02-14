//! React Query renderer for @aptx/react-query
//!
//! Generates React Query hooks using @aptx/react-query package.

use crate::query_base;

use swagger_gen::pipeline::{GeneratorInput, RenderOutput, Renderer};

/// React Query renderer for @aptx/react-query
///
/// Generates:
/// - Query files in `react-query/{namespace}/{operation}.query.ts`
/// - Mutation files in `react-query/{namespace}/{operation}.mutation.ts`
#[derive(Default)]
pub struct AptxReactQueryRenderer;

impl Renderer for AptxReactQueryRenderer {
    fn id(&self) -> &'static str {
        "aptx-react-query"
    }

    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String> {
        query_base::render_query_terminal(input, query_base::QueryTerminal::React)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_id() {
        let renderer = AptxReactQueryRenderer;
        assert_eq!(renderer.id(), "aptx-react-query");
    }

    #[test]
    fn test_renderer_with_empty_input() {
        let renderer = AptxReactQueryRenderer;
        let input = GeneratorInput {
            project: swagger_gen::pipeline::ProjectContext {
                package_name: "test".to_string(),
                api_base_path: None,
                terminals: vec![],
                retry_ownership: None,
            },
            endpoints: vec![],
            model_import: None,
            client_import: None,
            output_root: None,
        };

        let result = renderer.render(&input).unwrap();
        assert!(result.files.is_empty());
        assert!(result.warnings.is_empty());
    }
}
