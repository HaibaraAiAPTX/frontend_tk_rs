//! @aptx namespace commands for code generation.
//!
//! Provides commands for generating code using @aptx packages:
//! - `aptx:functions` - Generate function-style API calls
//! - `aptx:react-query` - Generate React Query hooks
//! - `aptx:vue-query` - Generate Vue Query composables

use std::path::Path;

use clap::Parser;
use swagger_gen::manifest::{generate_reports, update_manifest, ManifestTracker};
use swagger_gen::pipeline::{update_barrel_with_parents, CodegenPipeline, FileSystemWriter};
use swagger_gen_aptx::{
  AptxFunctionsRenderer, AptxMetaPass, AptxQueryMutationPass, AptxReactQueryRenderer,
  AptxVueQueryRenderer,
};
use swagger_tk::model::OpenAPIObject;

use super::output_lock::lock_output_root;

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

  #[arg(long)]
  model_mode: Option<String>,

  #[arg(long)]
  model_path: Option<String>,

  /// Disable manifest tracking
  #[arg(long, default_value = "false")]
  no_manifest: bool,

  /// Custom manifest directory (default: .generated)
  #[arg(long, default_value = ".generated")]
  manifest_dir: String,

  /// Preview mode: generate report without updating manifest
  #[arg(long, default_value = "false")]
  dry_run: bool,
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

/// Build model import configuration from command-line options
fn build_model_import_config(
  model_mode: Option<&str>,
  model_path: Option<&str>,
) -> Option<swagger_gen::pipeline::ModelImportConfig> {
  match model_mode {
    None if model_path.is_some() => Some(swagger_gen::pipeline::ModelImportConfig {
      import_type: "relative".to_string(),
      package_path: None,
      relative_path: None,
      original_path: model_path.map(|s| s.to_string()),
    }),
    None => None,
    Some("package") => Some(swagger_gen::pipeline::ModelImportConfig {
      import_type: "package".to_string(),
      package_path: Some(model_path.unwrap_or("@my-org/models").to_string()),
      relative_path: None,
      original_path: model_path.map(|s| s.to_string()),
    }),
    Some("relative") => Some(swagger_gen::pipeline::ModelImportConfig {
      import_type: "relative".to_string(),
      package_path: None,
      relative_path: None,
      original_path: model_path.map(|s| s.to_string()),
    }),
    _ => None,
  }
}

fn process_manifest_and_barrel(
  output: &Path,
  generator_id: &str,
  execution_plan: &swagger_gen::pipeline::ExecutionPlan,
  manifest_dir: &str,
  dry_run: bool,
) {
  let mut tracker = ManifestTracker::new(generator_id);
  for file in &execution_plan.planned_files {
    let name = manifest_entry_name(&file.path);
    tracker.track(name, file.path.clone());
  }

  let manifest_path = output.join(manifest_dir).join("manifest.json");
  let entries = tracker.entries().clone();
  let diff = tracker.finish(&manifest_path);

  if let Err(e) = generate_reports(&diff, output, manifest_dir) {
    eprintln!("Warning: Failed to generate reports: {}", e);
  }

  if !dry_run {
    if let Err(e) = update_manifest(&manifest_path, generator_id.to_string(), entries, "", "") {
      eprintln!("Warning: Failed to update manifest: {}", e);
    }
  }

  if let Err(e) = update_barrel_with_parents("", output) {
    eprintln!("Warning: Failed to update barrel files: {}", e);
  }

  if diff.has_changes() {
    println!("Manifest changes:");
    println!("  Added: {} files", diff.added.len());
    println!("  Deleted: {} files", diff.deleted.len());
    println!("  Unchanged: {} files", diff.unchanged.len());
  }
}

fn manifest_entry_name(path: &str) -> String {
  let normalized = path.replace('\\', "/");
  Path::new(&normalized)
    .with_extension("")
    .to_string_lossy()
    .replace('\\', "/")
}

fn run_aptx_codegen(
  args: &[String],
  open_api: &OpenAPIObject,
  command_name: &str,
  renderer: Box<dyn swagger_gen::pipeline::Renderer>,
) {
  let result = (|| -> Result<(), String> {
    let args: Vec<String> = std::iter::once("--".to_string())
      .chain(args.iter().cloned())
      .collect();
    let options =
      AptxCodegenOps::try_parse_from(args).map_err(|e| format!("Invalid arguments: {e}"))?;

    let output = Path::new(&options.output);
    let _output_lock = lock_output_root(output)?;

    let client_import = build_client_import_config(
      options.client_mode.as_deref(),
      options.client_path.as_deref(),
      options.client_package.as_deref(),
      options.client_import_name.as_deref(),
    );
    let model_import =
      build_model_import_config(options.model_mode.as_deref(), options.model_path.as_deref());

    let pipeline = CodegenPipeline::default()
      .with_transform(Box::new(AptxQueryMutationPass))
      .with_transform(Box::new(AptxMetaPass))
      .with_client_import(client_import)
      .with_model_import(model_import)
      .with_renderer(renderer)
      .with_writer(Box::new(FileSystemWriter::new(output)));

    let execution_plan = pipeline.plan(open_api)?;

    if !options.no_manifest {
      process_manifest_and_barrel(
        output,
        command_name,
        &execution_plan,
        &options.manifest_dir,
        options.dry_run,
      );
    }

    Ok(())
  })();

  if let Err(e) = result {
    panic!("{command_name} failed: {e}");
  }
}

/// Run aptx:functions command
pub fn run_aptx_functions(args: &[String], open_api: &OpenAPIObject) {
  run_aptx_codegen(
    args,
    open_api,
    "aptx:functions",
    Box::new(AptxFunctionsRenderer),
  );
}

/// Run aptx:react-query command
pub fn run_aptx_react_query(args: &[String], open_api: &OpenAPIObject) {
  run_aptx_codegen(
    args,
    open_api,
    "aptx:react-query",
    Box::new(AptxReactQueryRenderer),
  );
}

/// Run aptx:vue-query command
pub fn run_aptx_vue_query(args: &[String], open_api: &OpenAPIObject) {
  run_aptx_codegen(
    args,
    open_api,
    "aptx:vue-query",
    Box::new(AptxVueQueryRenderer),
  );
}

#[cfg(test)]
mod tests {
  use super::{build_model_import_config, manifest_entry_name};

  #[test]
  fn test_build_model_import_config_defaults_to_relative_when_only_model_path_is_provided() {
    let config =
      build_model_import_config(None, Some("./src/api/models")).expect("default relative config");

    assert_eq!(config.import_type, "relative");
    assert_eq!(config.original_path.as_deref(), Some("./src/api/models"));
    assert!(config.package_path.is_none());
  }

  #[test]
  fn test_build_model_import_config_relative() {
    let config = build_model_import_config(Some("relative"), Some("./src/api/models"))
      .expect("relative config");

    assert_eq!(config.import_type, "relative");
    assert_eq!(config.original_path.as_deref(), Some("./src/api/models"));
    assert!(config.package_path.is_none());
  }

  #[test]
  fn test_build_model_import_config_package() {
    let config =
      build_model_import_config(Some("package"), Some("@my-org/models")).expect("package config");

    assert_eq!(config.import_type, "package");
    assert_eq!(config.package_path.as_deref(), Some("@my-org/models"));
    assert_eq!(config.original_path.as_deref(), Some("@my-org/models"));
  }

  #[test]
  fn test_manifest_entry_name_keeps_directory_context() {
    assert_eq!(
      manifest_entry_name("spec/action_authority/add.ts"),
      "spec/action_authority/add"
    );
    assert_eq!(
      manifest_entry_name("functions/action_authority/index.ts"),
      "functions/action_authority/index"
    );
  }
}
