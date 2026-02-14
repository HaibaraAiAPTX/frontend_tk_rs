use std::time::Instant;
use swagger_tk::model::OpenAPIObject;

use super::{
    layout::{IdentityLayout, LayoutStrategy},
    model::{
        ClientImportConfig, ExecutionMetrics, ExecutionPlan, GeneratorInput, ModelImportConfig,
        RendererExecution,
    },
    parser::{OpenApiParser, Parser},
    renderer::{
        AxiosJsRenderer, AxiosTsRenderer, FunctionsRenderer, NoopRenderer, ReactQueryRenderer,
        Renderer, UniAppRenderer, VueQueryRenderer,
    },
    transform::{NormalizeEndpointPass, TransformPass},
    writer::{DryRunWriter, FileSystemWriter, Writer},
};

/// Convert client mode options to ClientImportConfig
fn build_client_import_config(
    client_mode: Option<&str>,
    client_path: Option<&str>,
    client_package: Option<&str>,
    client_import_name: Option<&str>,
) -> Option<ClientImportConfig> {
    match client_mode {
        None => None, // Default behavior
        Some(mode) => Some(ClientImportConfig {
            mode: mode.to_string(),
            client_path: client_path.map(|s| s.to_string()),
            client_package: client_package.map(|s| s.to_string()),
            import_name: client_import_name.map(|s| s.to_string()),
        }),
    }
}

/// Convert model mode options to ModelImportConfig
fn build_model_import_config(
    model_mode: Option<&str>,
    model_path: Option<&str>,
) -> Option<ModelImportConfig> {
    match model_mode {
        None => None,
        Some("package") => Some(ModelImportConfig {
            import_type: "package".to_string(),
            package_path: Some(model_path.unwrap_or("@my-org/models").to_string()),
            relative_path: None,
        }),
        Some("relative") => Some(ModelImportConfig {
            import_type: "relative".to_string(),
            package_path: None,
            relative_path: Some(model_path.unwrap_or("../../../spec/types").to_string()),
        }),
        _ => None,
    }
}

pub struct CodegenPipeline {
    parser: Box<dyn Parser>,
    transforms: Vec<Box<dyn TransformPass>>,
    renderers: Vec<Box<dyn Renderer>>,
    layout: Box<dyn LayoutStrategy>,
    writer: Box<dyn Writer>,
    client_import: Option<ClientImportConfig>,
    model_import: Option<ModelImportConfig>,
}

impl Default for CodegenPipeline {
    fn default() -> Self {
        Self {
            parser: Box::new(OpenApiParser),
            transforms: vec![Box::new(NormalizeEndpointPass)],
            renderers: vec![Box::new(NoopRenderer)],
            layout: Box::new(IdentityLayout),
            writer: Box::new(DryRunWriter),
            client_import: None,
            model_import: None,
        }
    }
}

impl CodegenPipeline {
    /// Set client import configuration
    pub fn with_client_import(mut self, config: Option<ClientImportConfig>) -> Self {
        self.client_import = config;
        self
    }

    /// Set model import configuration
    pub fn with_model_import(mut self, config: Option<ModelImportConfig>) -> Self {
        self.model_import = config;
        self
    }

    /// Add a renderer to the pipeline
    pub fn with_renderer(mut self, renderer: Box<dyn Renderer>) -> Self {
        // If we only have the default NoopRenderer, replace it
        if self.renderers.len() == 1 && self.renderers[0].id() == "noop" {
            self.renderers = vec![renderer];
        } else {
            self.renderers.push(renderer);
        }
        self
    }

    /// Set the writer for the pipeline
    pub fn with_writer(mut self, writer: Box<dyn Writer>) -> Self {
        self.writer = writer;
        self
    }
}

impl CodegenPipeline {
    pub fn functions_contract_v1(output_root: impl AsRef<std::path::Path>) -> Self {
        Self {
            parser: Box::new(OpenApiParser),
            transforms: vec![Box::new(NormalizeEndpointPass)],
            renderers: vec![Box::new(FunctionsRenderer)],
            layout: Box::new(IdentityLayout),
            writer: Box::new(FileSystemWriter::new(output_root)),
            client_import: None,
            model_import: None,
        }
    }

    pub fn react_query_contract_v1(output_root: impl AsRef<std::path::Path>) -> Self {
        Self {
            parser: Box::new(OpenApiParser),
            transforms: vec![Box::new(NormalizeEndpointPass)],
            renderers: vec![Box::new(ReactQueryRenderer)],
            layout: Box::new(IdentityLayout),
            writer: Box::new(FileSystemWriter::new(output_root)),
            client_import: None,
            model_import: None,
        }
    }

    pub fn vue_query_contract_v1(output_root: impl AsRef<std::path::Path>) -> Self {
        Self {
            parser: Box::new(OpenApiParser),
            transforms: vec![Box::new(NormalizeEndpointPass)],
            renderers: vec![Box::new(VueQueryRenderer)],
            layout: Box::new(IdentityLayout),
            writer: Box::new(FileSystemWriter::new(output_root)),
            client_import: None,
            model_import: None,
        }
    }

    pub fn axios_ts_v1(output_root: impl AsRef<std::path::Path>) -> Self {
        Self {
            parser: Box::new(OpenApiParser),
            transforms: vec![Box::new(NormalizeEndpointPass)],
            renderers: vec![Box::new(AxiosTsRenderer)],
            layout: Box::new(IdentityLayout),
            writer: Box::new(FileSystemWriter::new(output_root)),
            client_import: None,
            model_import: None,
        }
    }

    pub fn axios_js_v1(output_root: impl AsRef<std::path::Path>) -> Self {
        Self {
            parser: Box::new(OpenApiParser),
            transforms: vec![Box::new(NormalizeEndpointPass)],
            renderers: vec![Box::new(AxiosJsRenderer)],
            layout: Box::new(IdentityLayout),
            writer: Box::new(FileSystemWriter::new(output_root)),
            client_import: None,
            model_import: None,
        }
    }

    pub fn uniapp_v1(output_root: impl AsRef<std::path::Path>) -> Self {
        Self {
            parser: Box::new(OpenApiParser),
            transforms: vec![Box::new(NormalizeEndpointPass)],
            renderers: vec![Box::new(UniAppRenderer)],
            layout: Box::new(IdentityLayout),
            writer: Box::new(FileSystemWriter::new(output_root)),
            client_import: None,
            model_import: None,
        }
    }

    pub fn parse(&self, open_api: &OpenAPIObject) -> Result<GeneratorInput, String> {
        self.parser.parse(open_api)
    }

    pub fn plan(&self, open_api: &OpenAPIObject) -> Result<ExecutionPlan, String> {
        let total_start = Instant::now();
        let parse_start = Instant::now();
        let mut input = self.parser.parse(open_api)?;
        let parse_ms = parse_start.elapsed().as_millis();

        // Apply client_import configuration
        if let Some(ref client_import) = self.client_import {
            input.client_import = Some(client_import.clone());
        }
        if let Some(ref model_import) = self.model_import {
            input.model_import = Some(model_import.clone());
        }

        let transform_start = Instant::now();
        let mut transform_steps = Vec::new();
        for pass in &self.transforms {
            pass.apply(&mut input)?;
            transform_steps.push(pass.name().to_string());
        }
        let transform_ms = transform_start.elapsed().as_millis();

        let render_start = Instant::now();
        let mut renderer_reports = Vec::new();
        let mut planned_files = Vec::new();
        for renderer in &self.renderers {
            let output = renderer.render(&input)?;
            renderer_reports.push(RendererExecution {
                renderer_id: renderer.id().to_string(),
                planned_files: output.files.len(),
                warnings: output.warnings,
            });
            planned_files.extend(output.files);
        }
        let render_ms = render_start.elapsed().as_millis();

        let layout_start = Instant::now();
        let planned_files = self.layout.apply(planned_files);
        let layout_ms = layout_start.elapsed().as_millis();

        let write_start = Instant::now();
        let write_plan = self.writer.write(planned_files)?;
        let write_ms = write_start.elapsed().as_millis();

        Ok(ExecutionPlan {
            endpoint_count: input.endpoints.len(),
            transform_steps,
            renderer_reports,
            planned_files: write_plan.files_to_write,
            skipped_files: write_plan.skipped_files,
            metrics: ExecutionMetrics {
                parse_ms,
                transform_ms,
                render_ms,
                layout_ms,
                write_ms,
                total_ms: total_start.elapsed().as_millis(),
            },
        })
    }

    pub fn ir_snapshot_json(&self, open_api: &OpenAPIObject) -> Result<String, String> {
        let mut input = self.parser.parse(open_api)?;
        for pass in &self.transforms {
            pass.apply(&mut input)?;
        }
        serde_json::to_string_pretty(&input).map_err(|err| err.to_string())
    }
}

pub fn parse_openapi_to_ir(open_api: &OpenAPIObject) -> Result<GeneratorInput, String> {
    CodegenPipeline::default().parse(open_api)
}

pub fn build_dry_run_plan(open_api: &OpenAPIObject) -> Result<ExecutionPlan, String> {
    CodegenPipeline::default().plan(open_api)
}

pub fn build_report_json(open_api: &OpenAPIObject) -> Result<String, String> {
    let plan = build_dry_run_plan(open_api)?;
    serde_json::to_string_pretty(&plan).map_err(|err| err.to_string())
}

pub fn build_ir_snapshot_json(open_api: &OpenAPIObject) -> Result<String, String> {
    CodegenPipeline::default().ir_snapshot_json(open_api)
}

pub fn generate_functions_contract_v1(
    open_api: &OpenAPIObject,
    output_root: impl AsRef<std::path::Path>,
) -> Result<ExecutionPlan, String> {
    CodegenPipeline::functions_contract_v1(output_root).plan(open_api)
}

pub fn generate_functions_contract_v1_with_client(
    open_api: &OpenAPIObject,
    output_root: impl AsRef<std::path::Path>,
    client_mode: Option<&str>,
    client_path: Option<&str>,
    client_package: Option<&str>,
    client_import_name: Option<&str>,
) -> Result<ExecutionPlan, String> {
    let client_import =
        build_client_import_config(client_mode, client_path, client_package, client_import_name);
    CodegenPipeline::functions_contract_v1(output_root)
        .with_client_import(client_import)
        .plan(open_api)
}

pub fn generate_functions_contract_v1_with_imports(
    open_api: &OpenAPIObject,
    output_root: impl AsRef<std::path::Path>,
    client_mode: Option<&str>,
    client_path: Option<&str>,
    client_package: Option<&str>,
    client_import_name: Option<&str>,
    model_mode: Option<&str>,
    model_path: Option<&str>,
) -> Result<ExecutionPlan, String> {
    let client_import =
        build_client_import_config(client_mode, client_path, client_package, client_import_name);
    let model_import = build_model_import_config(model_mode, model_path);
    CodegenPipeline::functions_contract_v1(output_root)
        .with_client_import(client_import)
        .with_model_import(model_import)
        .plan(open_api)
}

pub fn generate_react_query_contract_v1(
    open_api: &OpenAPIObject,
    output_root: impl AsRef<std::path::Path>,
) -> Result<ExecutionPlan, String> {
    CodegenPipeline::react_query_contract_v1(output_root).plan(open_api)
}

pub fn generate_react_query_contract_v1_with_client(
    open_api: &OpenAPIObject,
    output_root: impl AsRef<std::path::Path>,
    client_mode: Option<&str>,
    client_path: Option<&str>,
    client_package: Option<&str>,
    client_import_name: Option<&str>,
) -> Result<ExecutionPlan, String> {
    let client_import =
        build_client_import_config(client_mode, client_path, client_package, client_import_name);
    CodegenPipeline::react_query_contract_v1(output_root)
        .with_client_import(client_import)
        .plan(open_api)
}

pub fn generate_react_query_contract_v1_with_imports(
    open_api: &OpenAPIObject,
    output_root: impl AsRef<std::path::Path>,
    client_mode: Option<&str>,
    client_path: Option<&str>,
    client_package: Option<&str>,
    client_import_name: Option<&str>,
    model_mode: Option<&str>,
    model_path: Option<&str>,
) -> Result<ExecutionPlan, String> {
    let client_import =
        build_client_import_config(client_mode, client_path, client_package, client_import_name);
    let model_import = build_model_import_config(model_mode, model_path);
    CodegenPipeline::react_query_contract_v1(output_root)
        .with_client_import(client_import)
        .with_model_import(model_import)
        .plan(open_api)
}

pub fn generate_vue_query_contract_v1(
    open_api: &OpenAPIObject,
    output_root: impl AsRef<std::path::Path>,
) -> Result<ExecutionPlan, String> {
    CodegenPipeline::vue_query_contract_v1(output_root).plan(open_api)
}

pub fn generate_vue_query_contract_v1_with_client(
    open_api: &OpenAPIObject,
    output_root: impl AsRef<std::path::Path>,
    client_mode: Option<&str>,
    client_path: Option<&str>,
    client_package: Option<&str>,
    client_import_name: Option<&str>,
) -> Result<ExecutionPlan, String> {
    let client_import =
        build_client_import_config(client_mode, client_path, client_package, client_import_name);
    CodegenPipeline::vue_query_contract_v1(output_root)
        .with_client_import(client_import)
        .plan(open_api)
}

pub fn generate_vue_query_contract_v1_with_imports(
    open_api: &OpenAPIObject,
    output_root: impl AsRef<std::path::Path>,
    client_mode: Option<&str>,
    client_path: Option<&str>,
    client_package: Option<&str>,
    client_import_name: Option<&str>,
    model_mode: Option<&str>,
    model_path: Option<&str>,
) -> Result<ExecutionPlan, String> {
    let client_import =
        build_client_import_config(client_mode, client_path, client_package, client_import_name);
    let model_import = build_model_import_config(model_mode, model_path);
    CodegenPipeline::vue_query_contract_v1(output_root)
        .with_client_import(client_import)
        .with_model_import(model_import)
        .plan(open_api)
}

pub fn generate_axios_ts_v1(
    open_api: &OpenAPIObject,
    output_root: impl AsRef<std::path::Path>,
) -> Result<ExecutionPlan, String> {
    CodegenPipeline::axios_ts_v1(output_root).plan(open_api)
}

pub fn generate_axios_js_v1(
    open_api: &OpenAPIObject,
    output_root: impl AsRef<std::path::Path>,
) -> Result<ExecutionPlan, String> {
    CodegenPipeline::axios_js_v1(output_root).plan(open_api)
}

pub fn generate_uniapp_v1(
    open_api: &OpenAPIObject,
    output_root: impl AsRef<std::path::Path>,
) -> Result<ExecutionPlan, String> {
    CodegenPipeline::uniapp_v1(output_root).plan(open_api)
}
