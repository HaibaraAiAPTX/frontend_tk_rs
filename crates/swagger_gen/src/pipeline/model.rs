use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorInput {
    pub project: ProjectContext,
    pub endpoints: Vec<EndpointItem>,
    pub model_import: Option<ModelImportConfig>,
    pub client_import: Option<ClientImportConfig>,
    /// Output root directory for generated files (used for calculating relative import paths)
    pub output_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelImportConfig {
    pub import_type: String,
    pub package_path: Option<String>,
    pub relative_path: Option<String>,
    /// Original model path as provided by user (before conversion to correct relative path)
    pub original_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientImportConfig {
    pub mode: String,
    pub client_path: Option<String>,
    pub client_package: Option<String>,
    pub import_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub package_name: String,
    pub api_base_path: Option<String>,
    pub terminals: Vec<String>,
    pub retry_ownership: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointItem {
    pub namespace: Vec<String>,
    pub operation_name: String,
    pub export_name: String,
    pub builder_name: String,
    pub summary: Option<String>,
    pub method: String,
    pub path: String,
    pub input_type_name: String,
    pub output_type_name: String,
    pub request_body_field: Option<String>,
    pub query_fields: Vec<String>,
    pub path_fields: Vec<String>,
    pub has_request_options: bool,
    pub supports_query: bool,
    pub supports_mutation: bool,
    pub deprecated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererExecution {
    pub renderer_id: String,
    pub planned_files: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub endpoint_count: usize,
    pub transform_steps: Vec<String>,
    pub renderer_reports: Vec<RendererExecution>,
    pub planned_files: Vec<PlannedFile>,
    pub skipped_files: usize,
    pub metrics: ExecutionMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub parse_ms: u128,
    pub transform_ms: u128,
    pub render_ms: u128,
    pub layout_ms: u128,
    pub write_ms: u128,
    pub total_ms: u128,
}

#[derive(Debug, Clone)]
pub struct RenderOutput {
    pub files: Vec<PlannedFile>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WritePlan {
    pub files_to_write: Vec<PlannedFile>,
    pub skipped_files: usize,
}
