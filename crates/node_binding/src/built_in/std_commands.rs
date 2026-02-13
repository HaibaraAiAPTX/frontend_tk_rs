//! Standard namespace commands for code generation.
//!
//! Provides commands for generating code using standard libraries:
//! - `std:axios-ts` - Generate TypeScript service classes with axios
//! - `std:axios-js` - Generate JavaScript functions with axios
//! - `std:uniapp` - Generate UniApp service classes

use std::path::Path;

use clap::Parser;
use swagger_gen::pipeline::{CodegenPipeline, FileSystemWriter};
use swagger_gen_standard::{AxiosJsRenderer, AxiosTsRenderer, UniAppRenderer};
use swagger_tk::model::OpenAPIObject;

/// Common options for std codegen commands
#[derive(Debug, Clone, Parser)]
pub struct StdCodegenOps {
    #[arg(long)]
    output: String,
}

/// Run std:axios-ts command
pub fn run_std_axios_ts(args: &[String], open_api: &OpenAPIObject) {
    let result = (|| -> Result<(), String> {
        let args: Vec<String> = std::iter::once("--".to_string())
            .chain(args.iter().cloned())
            .collect();
        let options = StdCodegenOps::try_parse_from(args)
            .map_err(|e| format!("Invalid arguments: {e}"))?;

        let output = Path::new(&options.output);

        let pipeline = CodegenPipeline::default()
            .with_renderer(Box::new(AxiosTsRenderer))
            .with_writer(Box::new(FileSystemWriter::new(output)));

        pipeline.plan(open_api)?;
        Ok(())
    })();

    if let Err(e) = result {
        panic!("std:axios-ts failed: {e}");
    }
}

/// Run std:axios-js command
pub fn run_std_axios_js(args: &[String], open_api: &OpenAPIObject) {
    let result = (|| -> Result<(), String> {
        let args: Vec<String> = std::iter::once("--".to_string())
            .chain(args.iter().cloned())
            .collect();
        let options = StdCodegenOps::try_parse_from(args)
            .map_err(|e| format!("Invalid arguments: {e}"))?;

        let output = Path::new(&options.output);

        let pipeline = CodegenPipeline::default()
            .with_renderer(Box::new(AxiosJsRenderer))
            .with_writer(Box::new(FileSystemWriter::new(output)));

        pipeline.plan(open_api)?;
        Ok(())
    })();

    if let Err(e) = result {
        panic!("std:axios-js failed: {e}");
    }
}

/// Run std:uniapp command
pub fn run_std_uniapp(args: &[String], open_api: &OpenAPIObject) {
    let result = (|| -> Result<(), String> {
        let args: Vec<String> = std::iter::once("--".to_string())
            .chain(args.iter().cloned())
            .collect();
        let options = StdCodegenOps::try_parse_from(args)
            .map_err(|e| format!("Invalid arguments: {e}"))?;

        let output = Path::new(&options.output);

        let pipeline = CodegenPipeline::default()
            .with_renderer(Box::new(UniAppRenderer))
            .with_writer(Box::new(FileSystemWriter::new(output)));

        pipeline.plan(open_api)?;
        Ok(())
    })();

    if let Err(e) = result {
        panic!("std:uniapp failed: {e}");
    }
}
