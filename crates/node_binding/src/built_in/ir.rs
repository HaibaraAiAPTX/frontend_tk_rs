use std::{fs, path::Path};

use aptx_frontend_tk_binding_plugin::utils::ensure_path;
use clap::Parser;
use swagger_gen::pipeline::build_ir_snapshot_json;
use swagger_tk::model::OpenAPIObject;

#[derive(Debug, Clone, Parser)]
pub struct IrSnapshotOps {
  #[arg(long)]
  output: String,
}

pub fn export_ir_snapshot(args: &[String], open_api: &OpenAPIObject) {
  let args: Vec<String> = std::iter::once("--".to_string())
    .chain(args.iter().cloned())
    .collect();
  let options = IrSnapshotOps::try_parse_from(args).unwrap();

  let output = Path::new(&options.output);
  if let Some(parent) = output.parent() {
    ensure_path(parent);
  }

  let json = build_ir_snapshot_json(open_api).unwrap();
  fs::write(output, json).unwrap();
}
