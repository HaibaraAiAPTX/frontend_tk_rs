//! Standard renderers for swagger_gen
//!
//! This crate provides renderer implementations that don't depend on
//! any specific business packages (@aptx or others).

// Re-exports from swagger_gen core
pub use swagger_gen::pipeline::{
    ClientImportConfig, EndpointItem, GeneratorInput, PlannedFile, RenderOutput, Renderer,
    normalize_type_ref, render_type_import_block, render_type_import_line,
};

pub use axios_ts::AxiosTsRenderer;
pub use axios_js::AxiosJsRenderer;
pub use uniapp::UniAppRenderer;

mod axios_ts;
mod axios_js;
mod uniapp;
