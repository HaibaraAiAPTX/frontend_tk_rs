//! @aptx namespace commands for code generation.
//!
//! Provides commands for generating code using @aptx packages:
//! - `aptx:functions` - Generate function-style API calls
//! - `aptx:react-query` - Generate React Query hooks
//! - `aptx:vue-query` - Generate Vue Query composables

use std::path::Path;

use clap::Parser;
use swagger_gen::pipeline::{CodegenPipeline, FileSystemWriter};
use swagger_gen_aptx::{AptxFunctionsRenderer, AptxReactQueryRenderer, AptxVueQueryRenderer};
use swagger_tk::model::OpenAPIObject;

/// Common options for @aptx codegen commands
#[derive(Debug, Clone, Parser)]
pub struct AptxCodegenOps {
    #[arg(long)]
    output: String,

    #[arg(long)]
    client_mode: Option<String>,

    #[arg(long)]
    client_path: Option<String>,

    #[arg(long)]
    client_package: Option<String>,

    #[arg(long)]
    client_import_name: Option<String>,
}

/// Run aptx:functions command
pub fn run_aptx_functions(args: &[String], open_api: &OpenAPIObject) {
    let result = (|| -> Result<(), String> {
        let args: Vec<String> = std::iter::once("--".to_string())
            .chain(args.iter().cloned())
            .collect();
        let options = AptxCodegenOps::try_parse_from(args)
            .map_err(|e| format!("Invalid arguments: {e}"))?;

        let output = Path::new(&options.output);

        let client_import = build_client_import_config(
            options.client_mode.as_deref(),
            options.client_path.as_deref(),
            options.client_package.as_deref(),
            options.client_import_name.as_deref(),
        );

        let pipeline = CodegenPipeline::default()
            .with_client_import(client_import)
            .with_renderer(Box::new(AptxFunctionsRenderer))
            .with_writer(Box::new(FileSystemWriter::new(output)));

        pipeline.plan(open_api)?;
        Ok(())
    })();

    if let Err(e) = result {
        panic!("aptx:functions failed: {e}");
    }
}

/// Run aptx:react-query command
pub fn run_aptx_react_query(args: &[String], open_api: &OpenAPIObject) {
    let result = (|| -> Result<(), String> {
        let args: Vec<String> = std::iter::once("--".to_string())
            .chain(args.iter().cloned())
            .collect();
        let options = AptxCodegenOps::try_parse_from(args)
            .map_err(|e| format!("Invalid arguments: {e}"))?;

        let output = Path::new(&options.output);

        let client_import = build_client_import_config(
            options.client_mode.as_deref(),
            options.client_path.as_deref(),
            options.client_package.as_deref(),
            options.client_import_name.as_deref(),
        );

        let pipeline = CodegenPipeline::default()
            .with_client_import(client_import)
            .with_renderer(Box::new(AptxReactQueryRenderer))
            .with_writer(Box::new(FileSystemWriter::new(output)));

        pipeline.plan(open_api)?;
        Ok(())
    })();

    if let Err(e) = result {
        panic!("aptx:react-query failed: {e}");
    }
}

/// Run aptx:vue-query command
pub fn run_aptx_vue_query(args: &[String], open_api: &OpenAPIObject) {
    let result = (|| -> Result<(), String> {
        let args: Vec<String> = std::iter::once("--".to_string())
            .chain(args.iter().cloned())
            .collect();
        let options = AptxCodegenOps::try_parse_from(args)
            .map_err(|e| format!("Invalid arguments: {e}"))?;

        let output = Path::new(&options.output);

        let client_import = build_client_import_config(
            options.client_mode.as_deref(),
            options.client_path.as_deref(),
            options.client_package.as_deref(),
            options.client_import_name.as_deref(),
        );

        let pipeline = CodegenPipeline::default()
            .with_client_import(client_import)
            .with_renderer(Box::new(AptxVueQueryRenderer))
            .with_writer(Box::new(FileSystemWriter::new(output)));

        pipeline.plan(open_api)?;
        Ok(())
    })();

    if let Err(e) = result {
        panic!("aptx:vue-query failed: {e}");
    }
}

/// Build client import configuration from command-line options
fn build_client_import_config(
    client_mode: Option<&str>,
    client_path: Option<&str>,
    client_package: Option<&str>,
    client_import_name: Option<&str>,
) -> Option<swagger_gen::pipeline::ClientImportConfig> {
    match client_mode {
        None => None,
        Some(mode) => Some(swagger_gen::pipeline::ClientImportConfig {
            mode: mode.to_string(),
            client_path: client_path.map(|s| s.to_string()),
            client_package: client_package.map(|s| s.to_string()),
            import_name: client_import_name.map(|s| s.to_string()),
        }),
    }
}
