//! Vue Query renderer for @aptx/vue-query
//!
//! Generates Vue Query composables using @aptx/vue-query package.

use crate::query_base;

use swagger_gen::pipeline::{GeneratorInput, RenderOutput, Renderer};

/// Vue Query renderer for @aptx/vue-query
///
/// Generates:
/// - Query files in `vue-query/{namespace}/{operation}.query.ts`
/// - Mutation files in `vue-query/{namespace}/{operation}.mutation.ts`
#[derive(Default)]
pub struct AptxVueQueryRenderer;

impl Renderer for AptxVueQueryRenderer {
    fn id(&self) -> &'static str {
        "aptx-vue-query"
    }

    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String> {
        query_base::render_query_terminal(input, query_base::QueryTerminal::Vue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_id() {
        let renderer = AptxVueQueryRenderer;
        assert_eq!(renderer.id(), "aptx-vue-query");
    }

    #[test]
    fn test_renderer_with_empty_input() {
        let renderer = AptxVueQueryRenderer;
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
