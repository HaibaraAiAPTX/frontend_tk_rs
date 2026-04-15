//! Python namespace commands for code generation.
//!
//! Provides commands for generating Python code:
//! - `python:functions` - Generate spec + function files using aptx_api_core
//! - `python:model` - Generate Pydantic model files
//! - `python:tools` - Generate tools.json for OpenAI function calling

use std::path::Path;

use clap::Parser;
use swagger_gen::manifest::{generate_reports, update_manifest, ManifestTracker};
use swagger_gen::pipeline::{CodegenPipeline, FileSystemWriter};
use swagger_gen_python::{PythonFunctionsRenderer, PythonToolsRenderer};
use swagger_tk::model::OpenAPIObject;

use super::output_lock::lock_output_root;

/// Common options for Python codegen commands
#[derive(Debug, Clone, Parser)]
pub struct PythonCodegenOps {
  #[arg(long)]
  output: String,

  #[arg(long)]
  model_mode: Option<String>,

  #[arg(long)]
  model_path: Option<String>,

  /// Disable manifest tracking
  #[arg(long, default_value = "false")]
  no_manifest: bool,

  #[arg(long, default_value = ".generated")]
  manifest_dir: String,

  #[arg(long, default_value = "false")]
  dry_run: bool,
}

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
      package_path: Some(model_path.unwrap_or("models").to_string()),
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

fn process_manifest(
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

fn run_python_codegen(
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
      PythonCodegenOps::try_parse_from(args).map_err(|e| format!("Invalid arguments: {e}"))?;

    let output = Path::new(&options.output);
    let _output_lock = lock_output_root(output)?;
    let model_import =
      build_model_import_config(options.model_mode.as_deref(), options.model_path.as_deref());

    let pipeline = CodegenPipeline::default()
      .with_model_import(model_import)
      .with_output_root(Some(output.to_string_lossy().to_string()))
      .with_renderer(renderer)
      .with_writer(Box::new(FileSystemWriter::new(output)));

    let execution_plan = pipeline.plan(open_api)?;

    if !options.no_manifest {
      process_manifest(
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

/// Run python:functions command
pub fn run_python_functions(args: &[String], open_api: &OpenAPIObject) {
  run_python_codegen(
    args,
    open_api,
    "python:functions",
    Box::new(PythonFunctionsRenderer),
  );
}

/// Run python:model command
pub fn run_python_model(args: &[String], open_api: &OpenAPIObject) {
  use aptx_frontend_tk_binding_plugin::utils::ensure_path;
  use std::fs;
  use swagger_gen::model_pipeline::parse_openapi_to_model_ir;
  use swagger_gen_python::render_pydantic_models;

  let result = (|| -> Result<(), String> {
    let args: Vec<String> = std::iter::once("--".to_string())
      .chain(args.iter().cloned())
      .collect();
    let options =
      PythonCodegenOps::try_parse_from(args).map_err(|e| format!("Invalid arguments: {e}"))?;

    let output = Path::new(&options.output);
    let _output_lock = lock_output_root(output)?;
    ensure_path(output);

    let ir = parse_openapi_to_model_ir(open_api)
      .map_err(|e| format!("Failed to parse OpenAPI to model IR: {e}"))?;

    let models =
      render_pydantic_models(&ir).map_err(|e| format!("Model generation failed: {e}"))?;

    let mut tracker = ManifestTracker::new("python:model");

    for (file_name, content) in &models {
      let file_path = output.join(file_name);
      if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
      }
      fs::write(&file_path, content).unwrap();
      let model_name = manifest_entry_name(file_name);
      tracker.track(model_name, file_name);
    }

    if !options.no_manifest {
      let manifest_path = output.join(&options.manifest_dir).join("manifest.json");
      let entries = tracker.entries().clone();
      let diff = tracker.finish(&manifest_path);

      if let Err(e) = generate_reports(&diff, output, &options.manifest_dir) {
        eprintln!("Warning: Failed to generate reports: {}", e);
      }

      if !options.dry_run {
        if let Err(e) = update_manifest(&manifest_path, "python:model".to_string(), entries, "", "")
        {
          eprintln!("Warning: Failed to update manifest: {}", e);
        }
      }

      if diff.has_changes() {
        println!("Manifest changes:");
        println!("  Added: {} files", diff.added.len());
        println!("  Deleted: {} files", diff.deleted.len());
        println!("  Unchanged: {} files", diff.unchanged.len());
      }
    }

    Ok(())
  })();

  if let Err(e) = result {
    panic!("python:model failed: {e}");
  }
}

/// Run python:tools command
pub fn run_python_tools(args: &[String], open_api: &OpenAPIObject) {
  run_python_codegen(
    args,
    open_api,
    "python:tools",
    Box::new(PythonToolsRenderer),
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
    let config = build_model_import_config(Some("package"), Some("my_app.generated.models"))
      .expect("package config");

    assert_eq!(config.import_type, "package");
    assert_eq!(
      config.package_path.as_deref(),
      Some("my_app.generated.models")
    );
    assert_eq!(
      config.original_path.as_deref(),
      Some("my_app.generated.models")
    );
  }

  #[test]
  fn test_manifest_entry_name_keeps_directory_context() {
    assert_eq!(
      manifest_entry_name("spec/action_authority/add_spec.py"),
      "spec/action_authority/add_spec"
    );
    assert_eq!(
      manifest_entry_name("functions/action_authority/__init__.py"),
      "functions/action_authority/__init__"
    );
  }
}
