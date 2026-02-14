//! Shared utilities for code generation renderers.
//!
//! This module provides common utility functions used by renderers
//! across different crates (swagger_gen_aptx, swagger_gen_standard).

mod import_utils;
mod type_utils;

pub use import_utils::{
    get_client_call, get_client_import_lines, get_model_import_base, resolve_file_import_path,
    resolve_model_import_base, should_use_package_import,
};

pub use type_utils::{
    is_identifier_type, is_primitive_type, normalize_type_ref, render_type_import_block,
    render_type_import_line,
};
