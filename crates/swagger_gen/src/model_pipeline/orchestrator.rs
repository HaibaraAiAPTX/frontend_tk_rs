use std::collections::HashMap;

use inflector::cases::pascalcase::to_pascal_case;
use swagger_tk::model::OpenAPIObject;

use super::{
    model::{
        EnumConflictPolicy, EnumPatch, ModelEnumMember, ModelEnumPlan, ModelEnumPlanItem,
        ModelEnumPlanMember, ModelIr, ModelKind, ModelLiteral, ModelRenderStyle,
    },
    parser::build_model_ir,
    renderer::render_model_files,
};

pub fn parse_openapi_to_model_ir(open_api: &OpenAPIObject) -> Result<ModelIr, String> {
    build_model_ir(open_api)
}

pub fn build_model_ir_snapshot_json(open_api: &OpenAPIObject) -> Result<String, String> {
    let ir = parse_openapi_to_model_ir(open_api)?;
    serde_json::to_string_pretty(&ir).map_err(|err| err.to_string())
}

pub fn build_model_enum_plan(
    open_api: &OpenAPIObject,
) -> Result<ModelEnumPlan, String> {
    build_model_enum_plan_with_existing(open_api, None)
}

pub fn build_model_enum_plan_with_existing(
    open_api: &OpenAPIObject,
    existing_enums: Option<&HashMap<String, HashMap<String, String>>>,
) -> Result<ModelEnumPlan, String> {
    let ir = parse_openapi_to_model_ir(open_api)?;
    Ok(build_model_enum_plan_from_ir(&ir, existing_enums))
}

pub fn build_model_enum_plan_json(open_api: &OpenAPIObject) -> Result<String, String> {
    build_model_enum_plan_json_with_existing(open_api, None)
}

pub fn build_model_enum_plan_json_with_existing(
    open_api: &OpenAPIObject,
    existing_enums: Option<&HashMap<String, HashMap<String, String>>>,
) -> Result<String, String> {
    let plan = build_model_enum_plan_with_existing(open_api, existing_enums)?;
    serde_json::to_string_pretty(&plan).map_err(|err| err.to_string())
}

pub fn generate_model_files(
    open_api: &OpenAPIObject,
    style: ModelRenderStyle,
    only_names: &[String],
) -> Result<HashMap<String, String>, String> {
    let ir = parse_openapi_to_model_ir(open_api)?;
    render_model_files(&ir, style, only_names)
}

pub fn generate_model_files_with_enum_patch(
    open_api: &OpenAPIObject,
    style: ModelRenderStyle,
    only_names: &[String],
    patches: &[EnumPatch],
    conflict_policy: EnumConflictPolicy,
) -> Result<HashMap<String, String>, String> {
    let mut ir = parse_openapi_to_model_ir(open_api)?;
    apply_enum_patches_to_ir(&mut ir, patches, conflict_policy)?;
    render_model_files(&ir, style, only_names)
}

fn build_model_enum_plan_from_ir(
    ir: &ModelIr,
    existing_enums: Option<&HashMap<String, HashMap<String, String>>>,
) -> ModelEnumPlan {
    let mut enums = Vec::new();
    for model in &ir.models {
        let ModelKind::Enum { members } = &model.kind else {
            continue;
        };

        // Get existing members for this enum if available
        let existing_members = existing_enums.and_then(|m| m.get(&model.name));

        enums.push(ModelEnumPlanItem {
            enum_name: model.name.clone(),
            description: model.description.clone(),
            source: "openapi".to_string(),
            members: members
                .iter()
                .enumerate()
                .map(|(index, member)| {
                    let value_key = literal_to_key(&member.value);
                    let mut name = member.name.clone();

                    // If we have existing data, check if we should preserve the historical name
                    if let Some(existing) = existing_members {
                        if let Some(historical_name) = existing.get(&value_key) {
                            // Only preserve if the historical name is NOT auto-generated
                            if !is_auto_generated(historical_name, &value_key, index) {
                                name = historical_name.clone();
                            }
                        }
                    }

                    ModelEnumPlanMember {
                        name,
                        value: value_key,
                        comment: member.comment.clone(),
                    }
                })
                .collect(),
        });
    }
    ModelEnumPlan {
        schema_version: "1".to_string(),
        enums,
    }
}

fn apply_enum_patches_to_ir(
    ir: &mut ModelIr,
    patches: &[EnumPatch],
    conflict_policy: EnumConflictPolicy,
) -> Result<(), String> {
    for patch in patches {
        let model = ir
            .models
            .iter_mut()
            .find(|model| model.name == patch.enum_name)
            .ok_or_else(|| format!("enum model not found: {}", patch.enum_name))?;
        let ModelKind::Enum { members } = &mut model.kind else {
            return Err(format!("model is not enum: {}", patch.enum_name));
        };
        apply_patch_to_enum_members(members, patch, conflict_policy);
    }
    Ok(())
}

fn apply_patch_to_enum_members(
    members: &mut Vec<ModelEnumMember>,
    patch: &EnumPatch,
    conflict_policy: EnumConflictPolicy,
) {
    for patch_member in &patch.members {
        if let Some(target) = members
            .iter_mut()
            .find(|member| literal_to_key(&member.value) == patch_member.value)
        {
            if conflict_policy != EnumConflictPolicy::OpenApiFirst {
                if let Some(suggested) = non_empty(&patch_member.suggested_name) {
                    target.name = suggested.to_string();
                }
            }
            if let Some(comment) = non_empty(&patch_member.comment) {
                target.comment = Some(comment.to_string());
            }
            continue;
        }

        let new_name = non_empty(&patch_member.suggested_name)
            .map(ToString::to_string)
            .unwrap_or_else(|| default_member_name(&patch_member.value, members.len() + 1));
        members.push(ModelEnumMember {
            name: new_name,
            value: parse_patch_member_value(&patch_member.value),
            comment: patch_member.comment.clone(),
        });
    }

    ensure_unique_member_names(members);
}

fn parse_patch_member_value(raw: &str) -> ModelLiteral {
    if raw.parse::<i64>().is_ok() {
        return ModelLiteral::Number {
            value: raw.to_string(),
        };
    }
    if raw.parse::<f64>().is_ok() {
        return ModelLiteral::Number {
            value: raw.to_string(),
        };
    }
    ModelLiteral::String {
        value: raw.to_string(),
    }
}

fn default_member_name(value: &str, index: usize) -> String {
    let sanitized = value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { ' ' })
        .collect::<String>();
    let pascal = to_pascal_case(sanitized.trim());
    if pascal.is_empty() {
        return format!("Value{index}");
    }
    if pascal
        .chars()
        .next()
        .map(|ch| ch.is_ascii_digit())
        .unwrap_or(false)
    {
        format!("Value{}", pascal)
    } else {
        pascal
    }
}

/// Check if a member name was auto-generated by the system.
/// This compares against both the simple parser naming (Value1, Value2)
/// and the complex orchestrator naming (from default_member_name).
fn is_auto_generated(name: &str, value: &str, index: usize) -> bool {
    // Parser naming: Value1, Value2, Value3, ... (index may change across OpenAPI updates)
    let parser_style_value = name
        .strip_prefix("Value")
        .map(|suffix| !suffix.is_empty() && suffix.chars().all(|ch| ch.is_ascii_digit()))
        .unwrap_or(false);
    let simple_candidate = format!("Value{}", index + 1);
    // Complex naming from default_member_name
    let complex_candidate = default_member_name(value, index + 1);

    parser_style_value || name == simple_candidate || name == complex_candidate
}

fn ensure_unique_member_names(members: &mut [ModelEnumMember]) {
    let mut used = std::collections::HashSet::<String>::new();
    for member in members.iter_mut() {
        let base = if member.name.is_empty() {
            "Value".to_string()
        } else {
            member.name.clone()
        };
        if !used.contains(&base) {
            used.insert(base.clone());
            member.name = base;
            continue;
        }
        let mut next = 2;
        loop {
            let candidate = format!("{base}{next}");
            if !used.contains(&candidate) {
                used.insert(candidate.clone());
                member.name = candidate;
                break;
            }
            next += 1;
        }
    }
}

fn literal_to_key(literal: &ModelLiteral) -> String {
    match literal {
        ModelLiteral::String { value } => value.to_string(),
        ModelLiteral::Number { value } => value.to_string(),
    }
}

fn non_empty(value: &Option<String>) -> Option<&str> {
    value.as_deref().and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}
