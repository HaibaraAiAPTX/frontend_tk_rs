use std::{collections::HashMap, fs, path::Path};

use aptx_frontend_tk_binding_plugin::utils::ensure_path;
use clap::Parser;
use swagger_gen::model_pipeline::{build_model_enum_plan_json, build_model_enum_plan_json_with_existing};
use swagger_tk::model::OpenAPIObject;

#[derive(Debug, Clone, Parser)]
pub struct ModelEnumPlanOps {
  #[arg(long)]
  output: String,

  #[arg(long)]
  model_output: Option<String>,
}

pub fn export_model_enum_plan(args: &[String], open_api: &OpenAPIObject) {
  let args: Vec<String> = std::iter::once("--".to_string())
    .chain(args.iter().cloned())
    .collect();
  let options = ModelEnumPlanOps::try_parse_from(args).unwrap();

  let output = Path::new(&options.output);
  if let Some(parent) = output.parent() {
    ensure_path(parent);
  }

  let model_dir = options
    .model_output
    .as_deref()
    .map(Path::new)
    .unwrap_or_else(|| output.parent().unwrap_or(output));
  let existing_enums = load_existing_enums_from_model_files(model_dir);
  let json = if let Some(existing) = existing_enums.as_ref() {
    build_model_enum_plan_json_with_existing(open_api, Some(existing)).unwrap()
  } else {
    build_model_enum_plan_json(open_api).unwrap()
  };
  fs::write(output, json).unwrap();
}

fn load_existing_enums_from_model_files(
  dir: &Path,
) -> Option<HashMap<String, HashMap<String, String>>> {
  let mut result = HashMap::<String, HashMap<String, String>>::new();
  let entries = fs::read_dir(dir).ok()?;
  for entry in entries.flatten() {
    let path = entry.path();
    if !path.is_file() {
      continue;
    }
    let Some(ext) = path.extension().and_then(|v| v.to_str()) else {
      continue;
    };
    if ext != "ts" {
      continue;
    }
    let Ok(content) = fs::read_to_string(&path) else {
      continue;
    };
    if let Some((enum_name, members)) = parse_typescript_enum_mapping(&content) {
      if !members.is_empty() {
        result.insert(enum_name, members);
      }
    }
  }
  if result.is_empty() {
    None
  } else {
    Some(result)
  }
}

fn parse_typescript_enum_mapping(content: &str) -> Option<(String, HashMap<String, String>)> {
  let enum_pos = content.find("export enum ")?;
  let header = &content[(enum_pos + "export enum ".len())..];
  let brace_pos = header.find('{')?;
  let enum_name = header[..brace_pos].trim().to_string();
  if enum_name.is_empty() {
    return None;
  }
  let body = &header[(brace_pos + 1)..];
  let end_pos = body.find('}')?;
  let body = &body[..end_pos];

  let mut members = HashMap::<String, String>::new();
  for line in body.lines() {
    let line = line.trim().trim_end_matches(',').trim();
    if line.is_empty()
      || line.starts_with("/**")
      || line.starts_with('*')
      || line.starts_with("*/")
      || line.starts_with("//")
    {
      continue;
    }
    let Some(eq_pos) = line.find('=') else {
      continue;
    };
    let name = line[..eq_pos].trim();
    let raw_value = line[(eq_pos + 1)..].trim();
    if name.is_empty() || raw_value.is_empty() {
      continue;
    }

    let value_key = if raw_value.starts_with('\"') {
      serde_json::from_str::<String>(raw_value).unwrap_or_else(|_| raw_value.to_string())
    } else {
      raw_value.to_string()
    };
    members.insert(value_key, name.to_string());
  }

  Some((enum_name, members))
}
