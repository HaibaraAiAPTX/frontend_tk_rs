//! Python code generation renderers for swagger_gen.
//!
//! Provides renderers that generate Python code from OpenAPI specifications:
//! - `PythonFunctionsRenderer`: Generates spec + function files using aptx_api_core
//! - `PydanticModelRenderer`: Generates Pydantic model files
//! - `PythonToolsRenderer`: Generates OpenAI function calling tools.json

pub use swagger_gen::pipeline::{EndpointItem, GeneratorInput, PlannedFile, RenderOutput, Renderer};
pub use swagger_gen::model_pipeline::{
    ModelIr, ModelKind, ModelType, ModelLiteral, ModelNode, ModelProperty, ModelEnumMember,
};

mod py_types;
mod pydantic_model;
mod python_functions;
mod python_tools;

pub use python_functions::PythonFunctionsRenderer;
pub use python_tools::PythonToolsRenderer;
pub use pydantic_model::render_pydantic_models;
