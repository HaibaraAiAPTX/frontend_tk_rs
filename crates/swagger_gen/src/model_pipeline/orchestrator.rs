use std::collections::HashMap;

use swagger_tk::model::OpenAPIObject;

use super::{
    model::{ModelIr, ModelRenderStyle},
    parser::build_model_ir,
    renderer::render_model_files,
};

pub fn parse_openapi_to_model_ir(open_api: &OpenAPIObject) -> Result<ModelIr, String> {
    build_model_ir(open_api)
}

pub fn build_model_ir_snapshot_json(open_api: &OpenAPIObject) -> Result<String, String> {
    let ir = parse_openapi_to_model_ir(open_api)?;
    serde_json::to_string_pretty(&ir).map_err(|err| err.to_string())
}

pub fn generate_model_files(
    open_api: &OpenAPIObject,
    style: ModelRenderStyle,
    only_names: &[String],
) -> Result<HashMap<String, String>, String> {
    let ir = parse_openapi_to_model_ir(open_api)?;
    render_model_files(&ir, style, only_names)
}

