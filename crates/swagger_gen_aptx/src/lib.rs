//! @aptx specific renderers for swagger_gen
//!
//! This crate provides code generation renderers that integrate with @aptx packages:
//! - `AptxFunctionsRenderer`: Generates function-style API calls using @aptx/api-client
//! - `AptxReactQueryRenderer`: Generates React Query hooks using @aptx/react-query
//! - `AptxVueQueryRenderer`: Generates Vue Query composables using @aptx/vue-query
//! - `AptxQueryMutationPass`: Custom query/mutation classification for @aptx APIs
//! - `AptxMetaPass`: Meta field configuration (e.g., skipAuthRefresh for refresh token endpoints)

pub use swagger_gen::pipeline::{
    ClientImportConfig, EndpointItem, GeneratorInput, PlannedFile, RenderOutput, Renderer,
    META_SUPPORTS_QUERY,
};

/// Meta key for skipping auth refresh (rendered to TS as RequestSpec meta field)
pub const META_SKIP_AUTH_REFRESH: &str = "SKIP_AUTH_REFRESH_META_KEY";

// Re-export utility functions from swagger_gen
// Note: These are re-exported from swagger_gen::pipeline via pub use utils::*;
pub use swagger_gen::pipeline::{
    get_client_call, get_client_import_lines, get_model_import_base, normalize_type_ref,
    render_type_import_block, render_type_import_line, resolve_file_import_path,
    resolve_model_import_base, should_use_package_import,
};

mod classifier;
mod functions;
mod meta_pass;
mod query_base;
mod react_query;
mod vue_query;

pub use classifier::AptxQueryMutationPass;
pub use functions::AptxFunctionsRenderer;
pub use meta_pass::AptxMetaPass;
pub use react_query::AptxReactQueryRenderer;
pub use vue_query::AptxVueQueryRenderer;
