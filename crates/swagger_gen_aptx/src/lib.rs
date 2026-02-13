//! @aptx specific renderers for swagger_gen
//!
//! This crate provides code generation renderers that integrate with @aptx packages:
//! - `AptxFunctionsRenderer`: Generates function-style API calls using @aptx/api-client
//! - `AptxReactQueryRenderer`: Generates React Query hooks using @aptx/react-query
//! - `AptxVueQueryRenderer`: Generates Vue Query composables using @aptx/vue-query

pub use swagger_gen::pipeline::{
    ClientImportConfig, EndpointItem, GeneratorInput, PlannedFile, RenderOutput, Renderer,
};

// Re-export utility functions from swagger_gen
// Note: These are re-exported from swagger_gen::pipeline via pub use utils::*;
pub use swagger_gen::pipeline::{
    get_client_call,
    get_client_import_lines,
    get_model_import_base,
    normalize_type_ref,
    render_type_import_block,
    render_type_import_line,
    should_use_package_import,
};

mod functions;
mod query_base;
mod react_query;
mod vue_query;

pub use functions::AptxFunctionsRenderer;
pub use react_query::AptxReactQueryRenderer;
pub use vue_query::AptxVueQueryRenderer;
