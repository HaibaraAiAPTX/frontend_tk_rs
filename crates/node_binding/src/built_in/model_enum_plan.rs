use std::{collections::HashMap, fs, path::Path};

use aptx_frontend_tk_binding_plugin::utils::ensure_path;
use clap::Parser;
use swagger_gen::model_pipeline::{
  build_model_enum_plan_json, build_model_enum_plan_json_with_existing, ExistingEnumMember,
};
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

pub fn load_existing_enums_from_model_files(
  dir: &Path,
) -> Option<HashMap<String, HashMap<String, ExistingEnumMember>>> {
  let mut result = HashMap::<String, HashMap<String, ExistingEnumMember>>::new();
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

fn parse_typescript_enum_mapping(
  content: &str,
) -> Option<(String, HashMap<String, ExistingEnumMember>)> {
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

  let mut members = HashMap::<String, ExistingEnumMember>::new();
  let mut pending_comment: Option<String> = None;

  for line in body.lines() {
    let line = line.trim();

    // Check for JSDoc comment: /** ... */
    if line.starts_with("/**") && line.ends_with("*/") {
      // Extract comment content between /** and */
      let comment_content = line[3..line.len() - 2].trim();
      if !comment_content.is_empty() {
        pending_comment = Some(comment_content.to_string());
      }
      continue;
    }

    // Skip other comment lines and empty lines
    if line.is_empty()
      || line.starts_with('*')
      || line.starts_with("*/")
      || line.starts_with("//")
      || line.starts_with("/*")
    {
      continue;
    }

    // Parse enum member: Name = value
    let line = line.trim_end_matches(',').trim();
    let Some(eq_pos) = line.find('=') else {
      // Not an enum member line, reset pending comment
      pending_comment = None;
      continue;
    };
    let name = line[..eq_pos].trim();
    let raw_value = line[(eq_pos + 1)..].trim();
    if name.is_empty() || raw_value.is_empty() {
      pending_comment = None;
      continue;
    }

    let value_key = if raw_value.starts_with('"') {
      serde_json::from_str::<String>(raw_value).unwrap_or_else(|_| raw_value.to_string())
    } else {
      raw_value.to_string()
    };

    members.insert(
      value_key,
      ExistingEnumMember {
        name: name.to_string(),
        comment: pending_comment.take(),
      },
    );
  }

  Some((enum_name, members))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_typescript_enum_mapping_should_extract_comments() {
    let content = r#"/** 任务状态 */
export enum AssignmentStatus {
  /** 启用 */
  Enabled = 0,
  /** 禁用 */
  Disabled = 1,
  /** 封禁 */
  Banned = 2,
}
"#;
    let result = parse_typescript_enum_mapping(content);
    assert!(result.is_some());
    let (enum_name, members) = result.unwrap();
    assert_eq!(enum_name, "AssignmentStatus");
    assert_eq!(members.len(), 3);

    let enabled = members.get("0").expect("Enabled should exist");
    assert_eq!(enabled.name, "Enabled");
    assert_eq!(enabled.comment, Some("启用".to_string()));

    let disabled = members.get("1").expect("Disabled should exist");
    assert_eq!(disabled.name, "Disabled");
    assert_eq!(disabled.comment, Some("禁用".to_string()));

    let banned = members.get("2").expect("Banned should exist");
    assert_eq!(banned.name, "Banned");
    assert_eq!(banned.comment, Some("封禁".to_string()));
  }

  #[test]
  fn parse_typescript_enum_mapping_should_handle_enum_without_comments() {
    let content = r#"export enum SimpleStatus {
  Active = 1,
  Inactive = 2,
}
"#;
    let result = parse_typescript_enum_mapping(content);
    assert!(result.is_some());
    let (enum_name, members) = result.unwrap();
    assert_eq!(enum_name, "SimpleStatus");
    assert_eq!(members.len(), 2);

    let active = members.get("1").expect("Active should exist");
    assert_eq!(active.name, "Active");
    assert_eq!(active.comment, None);

    let inactive = members.get("2").expect("Inactive should exist");
    assert_eq!(inactive.name, "Inactive");
    assert_eq!(inactive.comment, None);
  }

  #[test]
  fn parse_typescript_enum_mapping_should_handle_string_enum() {
    let content = r#"export enum StringEnum {
  /** 成功 */
  Success = "success",
  /** 失败 */
  Failed = "failed",
}
"#;
    let result = parse_typescript_enum_mapping(content);
    assert!(result.is_some());
    let (enum_name, members) = result.unwrap();
    assert_eq!(enum_name, "StringEnum");
    assert_eq!(members.len(), 2);

    let success = members.get("success").expect("Success should exist");
    assert_eq!(success.name, "Success");
    assert_eq!(success.comment, Some("成功".to_string()));
  }
}
