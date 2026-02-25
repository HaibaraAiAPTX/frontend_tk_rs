//! Renderer trait and implementations for code generation.
//!
//! This module defines the core `Renderer` trait.
//!
//! **Note**: The concrete renderer implementations have been moved to separate crates:
//! - `swagger_gen_aptx`: @aptx-specific renderers (AptxFunctionsRenderer, AptxReactQueryRenderer, AptxVueQueryRenderer)
//! - Standard renderers (AxiosTsRenderer, AxiosJsRenderer, UniAppRenderer) are also available in swagger_gen_aptx

use super::model::{GeneratorInput, RenderOutput};

/// Core trait for code generation renderers.
///
/// All renderers (regardless of which crate they're in) must implement this trait.
pub trait Renderer: Send + Sync {
    /// Returns a unique identifier for this renderer.
    fn id(&self) -> &'static str;

    /// Renders the generated code from the input.
    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String>;
}

/// A no-operation renderer that produces no output.
#[derive(Default)]
pub struct NoopRenderer;

impl Renderer for NoopRenderer {
    fn id(&self) -> &'static str {
        "noop"
    }

    fn render(&self, _input: &GeneratorInput) -> Result<RenderOutput, String> {
        Ok(RenderOutput {
            files: vec![],
            warnings: vec![],
        })
    }
}

