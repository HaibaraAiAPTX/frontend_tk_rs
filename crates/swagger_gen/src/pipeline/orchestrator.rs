use std::time::Instant;
use swagger_tk::model::OpenAPIObject;

use super::{
    layout::{IdentityLayout, LayoutStrategy},
    model::{ExecutionMetrics, ExecutionPlan, GeneratorInput, RendererExecution},
    parser::{OpenApiParser, Parser},
    renderer::{FunctionsRenderer, NoopRenderer, ReactQueryRenderer, Renderer, VueQueryRenderer},
    transform::{NormalizeEndpointPass, TransformPass},
    writer::{DryRunWriter, FileSystemWriter, Writer},
};

pub struct CodegenPipeline {
    parser: Box<dyn Parser>,
    transforms: Vec<Box<dyn TransformPass>>,
    renderers: Vec<Box<dyn Renderer>>,
    layout: Box<dyn LayoutStrategy>,
    writer: Box<dyn Writer>,
}

impl Default for CodegenPipeline {
    fn default() -> Self {
        Self {
            parser: Box::new(OpenApiParser),
            transforms: vec![Box::new(NormalizeEndpointPass)],
            renderers: vec![Box::new(NoopRenderer)],
            layout: Box::new(IdentityLayout),
            writer: Box::new(DryRunWriter),
        }
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
        }
    }

    pub fn react_query_contract_v1(output_root: impl AsRef<std::path::Path>) -> Self {
        Self {
            parser: Box::new(OpenApiParser),
            transforms: vec![Box::new(NormalizeEndpointPass)],
            renderers: vec![Box::new(ReactQueryRenderer)],
            layout: Box::new(IdentityLayout),
            writer: Box::new(FileSystemWriter::new(output_root)),
        }
    }

    pub fn vue_query_contract_v1(output_root: impl AsRef<std::path::Path>) -> Self {
        Self {
            parser: Box::new(OpenApiParser),
            transforms: vec![Box::new(NormalizeEndpointPass)],
            renderers: vec![Box::new(VueQueryRenderer)],
            layout: Box::new(IdentityLayout),
            writer: Box::new(FileSystemWriter::new(output_root)),
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

pub fn generate_react_query_contract_v1(
    open_api: &OpenAPIObject,
    output_root: impl AsRef<std::path::Path>,
) -> Result<ExecutionPlan, String> {
    CodegenPipeline::react_query_contract_v1(output_root).plan(open_api)
}

pub fn generate_vue_query_contract_v1(
    open_api: &OpenAPIObject,
    output_root: impl AsRef<std::path::Path>,
) -> Result<ExecutionPlan, String> {
    CodegenPipeline::vue_query_contract_v1(output_root).plan(open_api)
}
