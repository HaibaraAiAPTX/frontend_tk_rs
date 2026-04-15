use std::{fs, path::Path};

use aptx_frontend_tk_binding_plugin::utils::ensure_path;
use clap::Parser;
use swagger_gen::manifest::{generate_reports, update_manifest, ManifestTracker};
use swagger_gen::model_pipeline::{
  generate_model_files, generate_model_files_with_existing, ModelRenderStyle,
};
use swagger_tk::model::OpenAPIObject;

use super::model_enum_plan::load_existing_enums_from_model_files;
use super::output_lock::lock_output_root;

#[derive(Debug, Clone, Parser)]
pub struct ModelGenOps {
  #[arg(long)]
  output: String,

  #[arg(long, default_value = "module")]
  style: String,

  #[arg(long)]
  name: Option<Vec<String>>,

  #[arg(long, default_value = "false")]
  preserve: bool,

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

pub fn run_model_gen(args: &[String], open_api: &OpenAPIObject) {
  let args: Vec<String> = std::iter::once("--".to_string())
    .chain(args.iter().cloned())
    .collect();
  let options = ModelGenOps::try_parse_from(args).unwrap();
  let output = Path::new(&options.output);
  let _output_lock = lock_output_root(output).unwrap();
  ensure_path(output);
  let style = ModelRenderStyle::parse(&options.style).unwrap();
  let only_names = options.name.unwrap_or_default();

  // Create tracker
  let mut tracker = ManifestTracker::new("models");

  let models = if options.preserve {
    // Load existing enums from output directory
    let existing_enums = load_existing_enums_from_model_files(output);
    match existing_enums {
      Some(enums) => {
        generate_model_files_with_existing(open_api, style, &only_names, &enums).unwrap()
      }
      None => generate_model_files(open_api, style, &only_names).unwrap(),
    }
  } else {
    generate_model_files(open_api, style, &only_names).unwrap()
  };

  // Write files and track them
  // Note: `name` from render_model_files already includes the .ts or .d.ts suffix
  for (file_name, content) in &models {
    fs::write(output.join(file_name), content).unwrap();
    // Extract model name from file_name for tracking (remove .ts or .d.ts suffix)
    let model_name = file_name
      .strip_suffix(".ts")
      .or_else(|| file_name.strip_suffix(".d.ts"))
      .unwrap_or(file_name);
    tracker.track(model_name, file_name);
  }

  // Process manifest
  if !options.no_manifest {
    let manifest_path = output.join(&options.manifest_dir).join("manifest.json");

    // Clone entries before finish consumes the tracker
    let entries = tracker.entries().clone();

    // Calculate diff
    let diff = tracker.finish(&manifest_path);

    // Generate reports
    if let Err(e) = generate_reports(&diff, output, &options.manifest_dir) {
      eprintln!("Warning: Failed to generate reports: {}", e);
    }

    // Update manifest (non dry_run mode)
    if !options.dry_run {
      if let Err(e) = update_manifest(
        &manifest_path,
        "models".to_string(),
        entries,
        "", // openapi_hash
        "", // openapi_version
      ) {
        eprintln!("Warning: Failed to update manifest: {}", e);
      }
    }

    // Output summary
    if diff.has_changes() {
      println!("Manifest changes:");
      println!("  Added: {} files", diff.added.len());
      println!("  Deleted: {} files", diff.deleted.len());
      println!("  Unchanged: {} files", diff.unchanged.len());
    }
  }
}
