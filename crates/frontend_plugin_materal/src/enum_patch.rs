use std::{collections::HashSet, fs, thread, time::Duration};

use aptx_frontend_tk_binding_plugin::utils::ensure_path;
use clap::Parser;
use serde_json::Value;
use swagger_gen::model_pipeline::{EnumPatch, EnumPatchDocument, EnumPatchMember};
use swagger_tk::{getter::get_schema_by_name, model::OpenAPIObject};

#[derive(Debug, Clone, Parser)]
pub struct MateralEnumPatchOpts {
    #[arg(long)]
    base_url: String,

    #[arg(long)]
    output: String,

    #[arg(long, default_value_t = 3)]
    max_retries: usize,

    #[arg(long, default_value_t = 10_000)]
    timeout_ms: u64,

    #[arg(long, default_value = "auto")]
    naming_strategy: String,
}

pub fn export_materal_enum_patch(args: &[String], open_api: &OpenAPIObject) -> Result<(), String> {
    let args: Vec<String> = std::iter::once("--".to_string())
        .chain(args.iter().cloned())
        .collect();
    let opts = MateralEnumPatchOpts::try_parse_from(args).map_err(|err| err.to_string())?;

    let detected = detect_materal_enums(open_api);
    let mut patches = Vec::new();
    let naming_strategy = parse_naming_strategy(&opts.naming_strategy)?;

    for item in detected {
        let url = format!(
            "{}{}",
            opts.base_url.trim_end_matches('/'),
            item.path.as_str()
        );
        let values = fetch_enum_values(&url, opts.max_retries, opts.timeout_ms)?;
        let members = build_patch_members(&item.enum_name, values, naming_strategy);

        patches.push(EnumPatch {
            enum_name: item.enum_name.clone(),
            members,
            source: Some("materal-api".to_string()),
            confidence: Some(0.7),
        });
    }

    let output = std::path::Path::new(&opts.output);
    if let Some(parent) = output.parent() {
        ensure_path(parent);
    }

    let doc = EnumPatchDocument {
        schema_version: "1".to_string(),
        patches,
    };
    let text = serde_json::to_string_pretty(&doc).map_err(|err| err.to_string())?;
    fs::write(output, text).map_err(|err| err.to_string())?;
    Ok(())
}

struct MateralEnumEndpoint {
    enum_name: String,
    path: String,
}

fn detect_materal_enums(open_api: &OpenAPIObject) -> Vec<MateralEnumEndpoint> {
    let mut result = Vec::new();
    let Some(paths) = open_api.paths.as_ref() else {
        return result;
    };
    let mut path_keys = paths.keys().cloned().collect::<Vec<_>>();
    path_keys.sort();

    for path in path_keys {
        let Some(enum_name) = extract_enum_name(path.as_str()) else {
            continue;
        };

        let enum_name = if enum_name.is_empty() {
            continue;
        } else {
            enum_name
        };

        if get_schema_by_name(open_api, enum_name.as_str()).is_none() {
            continue;
        }

        result.push(MateralEnumEndpoint { enum_name, path });
    }
    result
}

fn extract_enum_name(path: &str) -> Option<String> {
    let marker = "/Enums/GetAll";
    let marker_index = path.find(marker)?;
    let suffix = &path[(marker_index + marker.len())..];
    let enum_name = suffix
        .split('/')
        .next()
        .map(str::trim)
        .unwrap_or_default()
        .to_string();
    if enum_name.is_empty() {
        None
    } else {
        Some(enum_name)
    }
}

fn fetch_enum_values(
    url: &str,
    max_retries: usize,
    timeout_ms: u64,
) -> Result<Vec<(String, String)>, String> {
    let retry_count = max_retries.max(1);
    let mut last_error = "unknown".to_string();
    for attempt in 1..=retry_count {
        match fetch_enum_values_once(url, timeout_ms) {
            Ok(values) => return Ok(values),
            Err(err) => {
                last_error = err;
                if attempt < retry_count {
                    let backoff = 1_u64 << (attempt - 1);
                    thread::sleep(Duration::from_millis(500 * backoff));
                }
            }
        }
    }
    Err(format!(
        "failed to fetch enum values from {url}: {last_error}"
    ))
}

fn fetch_enum_values_once(url: &str, timeout_ms: u64) -> Result<Vec<(String, String)>, String> {
    let response = ureq::get(url)
        .timeout(Duration::from_millis(timeout_ms))
        .call()
        .map_err(|err| err.to_string())?;
    let body = response.into_string().map_err(|err| err.to_string())?;
    let json = serde_json::from_str::<Value>(&body).map_err(|err| err.to_string())?;

    let data = json
        .get("Data")
        .and_then(Value::as_array)
        .ok_or_else(|| "response missing Data array".to_string())?;
    let mut result = Vec::new();
    for item in data {
        let key = item.get("Key").or_else(|| item.get("key"));
        let value = item.get("Value").or_else(|| item.get("value"));
        let raw_key = match key {
            Some(Value::String(v)) => v.clone(),
            Some(Value::Number(v)) => v.to_string(),
            Some(Value::Bool(v)) => v.to_string(),
            Some(other) => other.to_string(),
            None => continue,
        };
        let raw_value = match value {
            Some(Value::String(v)) => v.clone(),
            Some(Value::Number(v)) => v.to_string(),
            Some(Value::Bool(v)) => v.to_string(),
            Some(other) => other.to_string(),
            None => String::new(),
        };
        result.push((raw_key, raw_value));
    }
    Ok(result)
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NamingStrategy {
    Auto,
    None,
}

fn parse_naming_strategy(value: &str) -> Result<NamingStrategy, String> {
    match value {
        "auto" => Ok(NamingStrategy::Auto),
        "none" => Ok(NamingStrategy::None),
        _ => Err("`--naming-strategy` expects auto|none.".to_string()),
    }
}

fn build_patch_members(
    enum_name: &str,
    values: Vec<(String, String)>,
    naming_strategy: NamingStrategy,
) -> Vec<EnumPatchMember> {
    let mut used_names = HashSet::<String>::new();
    values
        .into_iter()
        .map(|(value, label)| {
            let comment = non_empty(label.clone());
            let suggested_name = if naming_strategy == NamingStrategy::Auto {
                Some(build_suggested_name(
                    enum_name,
                    &value,
                    comment.as_deref(),
                    &mut used_names,
                ))
            } else {
                None
            };
            EnumPatchMember {
                value,
                suggested_name,
                comment,
            }
        })
        .collect::<Vec<_>>()
}

fn build_suggested_name(
    enum_name: &str,
    value: &str,
    comment: Option<&str>,
    used_names: &mut HashSet<String>,
) -> String {
    let from_comment = comment.and_then(|v| {
        let pascal = sanitize_to_pascal(v);
        if pascal.is_empty() {
            None
        } else {
            Some(pascal)
        }
    });

    let base = if let Some(name) = from_comment {
        name
    } else {
        let key_suffix = sanitize_to_pascal(value);
        if key_suffix.is_empty() {
            format!("{enum_name}Value")
        } else {
            format!("{enum_name}{key_suffix}")
        }
    };

    ensure_unique_name(base, used_names)
}

fn ensure_unique_name(base: String, used_names: &mut HashSet<String>) -> String {
    if !used_names.contains(&base) {
        used_names.insert(base.clone());
        return base;
    }

    let mut index = 2;
    loop {
        let candidate = format!("{base}{index}");
        if !used_names.contains(&candidate) {
            used_names.insert(candidate.clone());
            return candidate;
        }
        index += 1;
    }
}

fn sanitize_to_pascal(raw: &str) -> String {
    let mut words = Vec::<String>::new();
    let mut current = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            current.push(ch);
        } else if !current.is_empty() {
            words.push(current.clone());
            current.clear();
        }
    }
    if !current.is_empty() {
        words.push(current);
    }

    let mut result = String::new();
    for word in words {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            result.push(first.to_ascii_uppercase());
            for rest in chars {
                result.push(rest.to_ascii_lowercase());
            }
        }
    }

    if result
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        format!("Value{result}")
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::{NamingStrategy, build_patch_members, extract_enum_name, parse_naming_strategy};

    #[test]
    fn extract_enum_name_should_work() {
        let name =
            extract_enum_name("/MainAPI/Enums/GetAllOrderStatus").expect("enum name required");
        assert_eq!(name, "OrderStatus");
    }

    #[test]
    fn extract_enum_name_should_ignore_non_materal_path() {
        assert!(extract_enum_name("/MainAPI/Orders/GetAll").is_none());
    }

    #[test]
    fn naming_strategy_parse_should_work() {
        assert_eq!(
            parse_naming_strategy("auto").expect("auto should parse"),
            NamingStrategy::Auto
        );
        assert_eq!(
            parse_naming_strategy("none").expect("none should parse"),
            NamingStrategy::None
        );
        assert!(parse_naming_strategy("x").is_err());
    }

    #[test]
    fn build_patch_members_should_create_suggested_names() {
        let members = build_patch_members(
            "Role",
            vec![
                ("0".to_string(), "Admin User".to_string()),
                ("1".to_string(), "Admin User".to_string()),
                ("2".to_string(), " ".to_string()),
            ],
            NamingStrategy::Auto,
        );
        assert_eq!(
            members[0]
                .suggested_name
                .clone()
                .expect("first name should exist"),
            "AdminUser".to_string()
        );
        assert_eq!(
            members[1]
                .suggested_name
                .clone()
                .expect("second name should exist"),
            "AdminUser2".to_string()
        );
        assert_eq!(
            members[2]
                .suggested_name
                .clone()
                .expect("fallback name should exist"),
            "RoleValue2".to_string()
        );
    }
}
