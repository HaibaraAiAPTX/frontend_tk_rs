use std::time::Instant;
use swagger_tk::model::OpenAPIObject;

use super::{
    layout::{IdentityLayout, LayoutStrategy, inject_barrel_indexes},
    model::{
        ClientImportConfig, ExecutionMetrics, ExecutionPlan, GeneratorInput, ModelImportConfig,
        RendererExecution,
    },
    parser::{OpenApiParser, Parser},
    renderer::{NoopRenderer, Renderer},
    transform::{NormalizeEndpointPass, TransformPass},
    writer::{DryRunWriter, Writer},
};

pub struct CodegenPipeline {
    parser: Box<dyn Parser>,
    transforms: Vec<Box<dyn TransformPass>>,
    renderers: Vec<Box<dyn Renderer>>,
    layout: Box<dyn LayoutStrategy>,
    writer: Box<dyn Writer>,
    client_import: Option<ClientImportConfig>,
    model_import: Option<ModelImportConfig>,
    /// Output root directory (used for calculating relative import paths)
    output_root: Option<String>,
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
            output_root: None,
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

    /// Set output root directory (used for calculating relative import paths)
    pub fn with_output_root(mut self, output_root: Option<String>) -> Self {
        self.output_root = output_root;
        self
    }
}

impl CodegenPipeline {
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
        // Apply output_root for relative path calculation.
        // Prefer explicit configuration; otherwise infer from writer.
        input.output_root = self.output_root.clone().or_else(|| {
            self.writer
                .output_root()
                .map(|path| path.to_string_lossy().to_string())
        });

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
        let planned_files = inject_barrel_indexes(planned_files);
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

