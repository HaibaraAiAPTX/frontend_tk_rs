//! Pydantic model renderer for Python code generation.

use std::collections::HashMap;

use swagger_gen::model_pipeline::{
    ModelEnumMember, ModelIr, ModelKind, ModelLiteral, ModelNode, ModelProperty, ModelType,
};

use crate::py_types::{collect_python_imports, render_python_type_nullable, to_snake_case};

/// Render all models in the IR to Pydantic Python files.
/// Returns a map of filename -> file content.
pub fn render_pydantic_models(ir: &ModelIr) -> Result<HashMap<String, String>, String> {
    let mut files = HashMap::new();

    // Collect all model types for import resolution
    let mut all_imports = Vec::new();

    for model in &ir.models {
        let model_imports = collect_model_imports(model);
        for imp in model_imports {
            if !all_imports.contains(&imp) {
                all_imports.push(imp);
            }
        }
    }

    for model in &ir.models {
        let file_name = format!("models/{}.py", model.name);
        let content = render_single_model(model)?;
        files.insert(file_name, content);
    }

    Ok(files)
}

fn render_single_model(model: &ModelNode) -> Result<String, String> {
    let mut imports = vec!["from __future__ import annotations".to_string()];

    let body = match &model.kind {
        ModelKind::Interface { properties } => {
            render_interface(model, properties, &mut imports)
        }
        ModelKind::Enum { members } => render_enum(model, members, &mut imports),
        ModelKind::Alias { target, nullable } => render_alias(model, target, *nullable, &mut imports),
    };

    // Collect type-level imports
    let type_imports = collect_model_type_imports(model);
    for imp in type_imports {
        if !imports.contains(&imp) {
            imports.push(imp);
        }
    }

    let model_imports = collect_model_ref_imports(model);
    for imp in model_imports {
        if !imports.contains(&imp) {
            imports.push(imp);
        }
    }

    // Always need these for interface models
    if matches!(model.kind, ModelKind::Interface { .. }) {
        let pydantic_imports = vec![
            "from pydantic import BaseModel, ConfigDict, Field",
        ];
        for imp in pydantic_imports {
            if !imports.contains(&imp.to_string()) {
                imports.push(imp.to_string());
            }
        }
    }

    let mut result = imports.join("\n");
    result.push_str("\n\n");
    result.push_str(&body);
    result.push('\n');

    Ok(result)
}

fn render_interface(
    model: &ModelNode,
    properties: &[ModelProperty],
    _imports: &mut Vec<String>,
) -> String {
    let mut lines = vec![format!("class {}(BaseModel):", model.name)];
    lines.push("    model_config = ConfigDict(populate_by_name=True)".to_string());
    lines.push(String::new());

    if properties.is_empty() {
        lines.push("    pass".to_string());
    } else {
        for prop in properties {
            let snake = to_snake_case(&prop.name);
            let py_type = render_python_type_nullable(&prop.r#type, prop.nullable || !prop.required);

            if prop.required && !prop.nullable {
                lines.push(format!("    {}: {} = Field(alias=\"{}\")", snake, py_type, prop.name));
            } else {
                lines.push(format!("    {}: {} = Field(default=None, alias=\"{}\")", snake, py_type, prop.name));
            }
        }
    }

    lines.join("\n")
}

fn render_enum(
    model: &ModelNode,
    members: &[ModelEnumMember],
    _imports: &mut Vec<String>,
) -> String {
    // Determine base type from first member
    let base_type = if let Some(first) = members.first() {
        match &first.value {
            ModelLiteral::String { .. } => "str",
            ModelLiteral::Number { .. } => "int",
        }
    } else {
        "str"
    };

    // Import Enum from enum module
    let enum_import = "from enum import Enum";

    let mut lines = vec![format!("class {}({}, Enum):", model.name, base_type)];

    if members.is_empty() {
        lines.push("    pass".to_string());
    } else {
        for member in members {
            let value = match &member.value {
                ModelLiteral::String { value } => format!("\"{}\"", value),
                ModelLiteral::Number { value } => value.clone(),
            };
            lines.push(format!("    {} = {}", member.name, value));
        }
    }

    // Wrap with enum import at top
    let mut result = String::new();
    result.push_str(enum_import);
    result.push_str("\n\n");
    result.push_str(&lines.join("\n"));
    result

}

fn render_alias(
    model: &ModelNode,
    target: &ModelType,
    nullable: bool,
    _imports: &mut Vec<String>,
) -> String {
    let py_type = render_python_type_nullable(target, nullable);
    format!("{} = {}", model.name, py_type)
}

fn collect_model_imports(model: &ModelNode) -> Vec<String> {
    match &model.kind {
        ModelKind::Interface { properties } => {
            let mut imports = Vec::new();
            for prop in properties {
                let mut prop_imports = collect_python_imports(&prop.r#type);
                imports.append(&mut prop_imports);
            }
            imports
        }
        ModelKind::Alias { target, .. } => collect_python_imports(target),
        _ => vec![],
    }
}

fn collect_model_type_imports(model: &ModelNode) -> Vec<String> {
    collect_model_imports(model)
}

fn collect_model_ref_imports(model: &ModelNode) -> Vec<String> {
    let mut refs = Vec::new();

    match &model.kind {
        ModelKind::Interface { properties } => {
            for prop in properties {
                collect_model_refs_recursive(&prop.r#type, &mut refs);
            }
        }
        ModelKind::Alias { target, .. } => collect_model_refs_recursive(target, &mut refs),
        ModelKind::Enum { .. } => {}
    }

    refs.sort();
    refs.dedup();

    refs.into_iter()
        .filter(|name| name != &model.name)
        .map(|name| format!("from .{name} import {name}"))
        .collect()
}

fn collect_model_refs_recursive(model_type: &ModelType, refs: &mut Vec<String>) {
    match model_type {
        ModelType::Ref { name } => refs.push(name.clone()),
        ModelType::Array { item } => collect_model_refs_recursive(item, refs),
        ModelType::Union { variants } => {
            for variant in variants {
                collect_model_refs_recursive(variant, refs);
            }
        }
        ModelType::String
        | ModelType::Number
        | ModelType::Boolean
        | ModelType::Object
        | ModelType::Literal { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_interface_model(name: &str, properties: Vec<ModelProperty>) -> ModelNode {
        ModelNode {
            name: name.to_string(),
            description: None,
            kind: ModelKind::Interface { properties },
        }
    }

    fn make_enum_model(name: &str, members: Vec<ModelEnumMember>) -> ModelNode {
        ModelNode {
            name: name.to_string(),
            description: None,
            kind: ModelKind::Enum { members },
        }
    }

    fn make_alias_model(name: &str, target: ModelType, nullable: bool) -> ModelNode {
        ModelNode {
            name: name.to_string(),
            description: None,
            kind: ModelKind::Alias { target, nullable },
        }
    }

    #[test]
    fn test_interface_rendering() {
        let model = make_interface_model("User", vec![
            ModelProperty {
                name: "id".to_string(),
                description: None,
                required: true,
                nullable: false,
                r#type: ModelType::String,
            },
            ModelProperty {
                name: "email".to_string(),
                description: None,
                required: false,
                nullable: true,
                r#type: ModelType::String,
            },
        ]);

        let content = render_single_model(&model).unwrap();
        assert!(content.contains("class User(BaseModel):"));
        assert!(content.contains("id: str = Field(alias=\"id\")"));
        assert!(content.contains("email: str | None = Field(default=None, alias=\"email\")"));
        assert!(content.contains("from __future__ import annotations"));
        assert!(content.contains("from pydantic import BaseModel, ConfigDict, Field"));
    }

    #[test]
    fn test_enum_str_rendering() {
        let model = make_enum_model("Status", vec![
            ModelEnumMember {
                name: "Active".to_string(),
                value: ModelLiteral::String { value: "active".to_string() },
                comment: None,
            },
            ModelEnumMember {
                name: "Inactive".to_string(),
                value: ModelLiteral::String { value: "inactive".to_string() },
                comment: None,
            },
        ]);

        let content = render_single_model(&model).unwrap();
        assert!(content.contains("class Status(str, Enum):"));
        assert!(content.contains("Active = \"active\""));
        assert!(content.contains("Inactive = \"inactive\""));
    }

    #[test]
    fn test_enum_int_rendering() {
        let model = make_enum_model("Priority", vec![
            ModelEnumMember {
                name: "Low".to_string(),
                value: ModelLiteral::Number { value: "1".to_string() },
                comment: None,
            },
            ModelEnumMember {
                name: "High".to_string(),
                value: ModelLiteral::Number { value: "3".to_string() },
                comment: None,
            },
        ]);

        let content = render_single_model(&model).unwrap();
        assert!(content.contains("class Priority(int, Enum):"));
        assert!(content.contains("Low = 1"));
        assert!(content.contains("High = 3"));
    }

    #[test]
    fn test_alias_rendering() {
        let model = make_alias_model("UserId", ModelType::String, false);
        let content = render_single_model(&model).unwrap();
        assert!(content.contains("UserId = str"));
    }

    #[test]
    fn test_alias_nullable() {
        let model = make_alias_model("MaybeUser", ModelType::Ref { name: "User".to_string() }, true);
        let content = render_single_model(&model).unwrap();
        assert!(content.contains("MaybeUser = User | None"));
    }

    #[test]
    fn test_snake_case_property_names() {
        let model = make_interface_model("Item", vec![
            ModelProperty {
                name: "firstName".to_string(),
                description: None,
                required: true,
                nullable: false,
                r#type: ModelType::String,
            },
            ModelProperty {
                name: "lastName".to_string(),
                description: None,
                required: true,
                nullable: false,
                r#type: ModelType::String,
            },
        ]);

        let content = render_single_model(&model).unwrap();
        assert!(content.contains("first_name: str = Field(alias=\"firstName\")"));
        assert!(content.contains("last_name: str = Field(alias=\"lastName\")"));
    }

    #[test]
    fn test_optional_field_default_none() {
        let model = make_interface_model("Config", vec![
            ModelProperty {
                name: "timeout".to_string(),
                description: None,
                required: false,
                nullable: true,
                r#type: ModelType::Number,
            },
        ]);

        let content = render_single_model(&model).unwrap();
        assert!(content.contains("timeout: float | None = Field(default=None, alias=\"timeout\")"));
    }

    #[test]
    fn test_object_type_imports_any() {
        let model = make_interface_model("Data", vec![
            ModelProperty {
                name: "payload".to_string(),
                description: None,
                required: true,
                nullable: false,
                r#type: ModelType::Object,
            },
        ]);

        let content = render_single_model(&model).unwrap();
        assert!(content.contains("from typing import Any"));
    }

    #[test]
    fn test_interface_imports_referenced_models() {
        let model = make_interface_model("UserDto", vec![
            ModelProperty {
                name: "status".to_string(),
                description: None,
                required: true,
                nullable: false,
                r#type: ModelType::Ref {
                    name: "Status".to_string(),
                },
            },
            ModelProperty {
                name: "roles".to_string(),
                description: None,
                required: false,
                nullable: false,
                r#type: ModelType::Array {
                    item: Box::new(ModelType::Ref {
                        name: "Role".to_string(),
                    }),
                },
            },
        ]);

        let content = render_single_model(&model).unwrap();
        assert!(content.contains("from .Role import Role"));
        assert!(content.contains("from .Status import Status"));
    }
}
