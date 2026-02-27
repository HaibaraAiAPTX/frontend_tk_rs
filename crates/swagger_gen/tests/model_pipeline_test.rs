use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use swagger_gen::model_pipeline::{
    build_model_enum_plan_json, build_model_enum_plan_json_with_existing, build_model_ir_snapshot_json,
    generate_model_files, generate_model_files_with_enum_patch, generate_model_files_with_existing,
    parse_openapi_to_model_ir, EnumConflictPolicy, EnumPatch, EnumPatchMember, ExistingEnumMember,
    ModelRenderStyle,
};
use swagger_tk::model::OpenAPIObject;

/// Mock OpenAPI spec with Order, User interfaces and OrderStatus enum for testing
const MOCK_OPENAPI: &str = r###"
{
  "openapi": "3.1.0",
  "info": { "title": "mock-api", "version": "1.0.0" },
  "paths": {},
  "components": {
    "schemas": {
      "Order": {
        "type": "object",
        "description": "Order information",
        "properties": {
          "id": { "type": "integer" },
          "userId": { "type": "integer" },
          "status": { "$ref": "#/components/schemas/OrderStatus" },
          "createdAt": { "type": "string", "format": "date-time" }
        },
        "required": ["id", "userId", "status"]
      },
      "User": {
        "type": "object",
        "description": "User information",
        "properties": {
          "id": { "type": "integer" },
          "name": { "type": "string" },
          "email": { "type": "string" }
        },
        "required": ["id", "name"]
      },
      "OrderStatus": {
        "type": "integer",
        "description": "Order status enum",
        "enum": [0, 1, 2, 3]
      }
    }
  }
}
"###;

fn get_mock_openapi() -> OpenAPIObject {
    OpenAPIObject::from_str(MOCK_OPENAPI).expect("parse mock openapi fail")
}

#[test]
fn parse_openapi_to_model_ir_test() {
    let open_api_object = get_mock_openapi();
    let ir = parse_openapi_to_model_ir(&open_api_object).expect("parse openapi to model ir fail");
    assert!(!ir.models.is_empty());
    assert!(ir.models.iter().any(|model| model.name == "Order"));
    assert!(ir.models.iter().any(|model| model.name == "User"));
    assert!(ir.models.iter().any(|model| model.name == "OrderStatus"));
}

#[test]
fn build_model_ir_snapshot_json_test() {
    let open_api_object = get_mock_openapi();
    let snapshot =
        build_model_ir_snapshot_json(&open_api_object).expect("build model ir snapshot fail");
    assert!(snapshot.contains("\"models\""));
    assert!(snapshot.contains("\"name\": \"Order\""));
    assert!(snapshot.contains("\"name\": \"User\""));
}

#[test]
fn build_model_enum_plan_json_test() {
    let open_api_object = get_mock_openapi();
    let snapshot = build_model_enum_plan_json(&open_api_object).expect("build enum plan fail");
    assert!(snapshot.contains("\"schema_version\": \"1\""));
    assert!(snapshot.contains("\"enums\""));
    assert!(snapshot.contains("\"enum_name\": \"OrderStatus\""));
}

#[test]
fn generate_model_files_with_two_styles_test() {
    let open_api_object = get_mock_openapi();

    // Test declaration style
    // Note: In declaration style, interfaces use .d.ts but enums use .ts
    let declaration_files =
        generate_model_files(&open_api_object, ModelRenderStyle::Declaration, &[])
            .expect("generate declaration files fail");
    assert!(declaration_files.contains_key("Order.d.ts"));
    assert!(declaration_files.contains_key("User.d.ts"));
    assert!(declaration_files.contains_key("OrderStatus.ts")); // enums use .ts in declaration style

    // Test module style
    let module_files = generate_model_files(&open_api_object, ModelRenderStyle::Module, &[])
        .expect("generate module files fail");
    let order_module = module_files
        .get("Order.ts")
        .expect("Order.ts should be generated");
    assert!(order_module.contains("export interface Order"));
    assert!(order_module.contains("id: number"));
    assert!(order_module.contains("userId: number"));
}

#[test]
fn generate_model_files_with_name_filter_test() {
    let open_api_object = get_mock_openapi();
    let files = generate_model_files(
        &open_api_object,
        ModelRenderStyle::Module,
        &["Order".to_string(), "User".to_string()],
    )
    .expect("generate filtered model files fail");

    assert_eq!(files.len(), 2);
    assert!(files.contains_key("Order.ts"));
    assert!(files.contains_key("User.ts"));

    let output_root = get_temp_test_dir("model-pipeline");
    fs::create_dir_all(&output_root).expect("create temp dir fail");
    for (name, content) in files {
        fs::write(output_root.join(name), content).expect("write file fail");
    }
    fs::remove_dir_all(&output_root).expect("remove temp dir fail");
}

#[test]
fn generate_model_files_with_enum_patch_test() {
    let open_api_object = OpenAPIObject::from_str(
        r#"
{
  "openapi": "3.1.0",
  "info": { "title": "enum-test", "version": "1.0.0" },
  "paths": {},
  "components": {
    "schemas": {
      "OrderStatus": {
        "type": "integer",
        "enum": [1, 2, 3]
      }
    }
  }
}
"#,
    )
    .expect("build openapi object for enum patch test fail");

    let patches = vec![EnumPatch {
        enum_name: "OrderStatus".to_string(),
        members: vec![
            EnumPatchMember {
                value: "1".to_string(),
                suggested_name: Some("PendingPayment".to_string()),
                comment: Some("待支付".to_string()),
            },
            EnumPatchMember {
                value: "99".to_string(),
                suggested_name: Some("CanceledBySystem".to_string()),
                comment: Some("系统取消".to_string()),
            },
        ],
        source: Some("test".to_string()),
        confidence: Some(1.0),
    }];

    let files = generate_model_files_with_enum_patch(
        &open_api_object,
        ModelRenderStyle::Module,
        &["OrderStatus".to_string()],
        &patches,
        EnumConflictPolicy::PatchFirst,
    )
    .expect("generate patched model files fail");

    let enum_file = files
        .get("OrderStatus.ts")
        .expect("patched enum file should be generated");
    assert!(enum_file.contains("PendingPayment"));
    assert!(enum_file.contains("CanceledBySystem = 99"));
    assert!(enum_file.contains("/** 待支付 */"));
}

#[test]
fn build_model_enum_plan_json_with_existing_should_preserve_translated_name() {
    let open_api_object = OpenAPIObject::from_str(
        r#"
{
  "openapi": "3.1.0",
  "info": { "title": "enum-test", "version": "1.0.0" },
  "paths": {},
  "components": {
    "schemas": {
      "OrderStatus": {
        "type": "string",
        "enum": ["成功", "失败"]
      }
    }
  }
}
"#,
    )
    .expect("build openapi object fail");

    let mut members = HashMap::new();
    members.insert(
        "成功".to_string(),
        ExistingEnumMember {
            name: "Success".to_string(),
            comment: None,
        },
    );
    members.insert(
        "失败".to_string(),
        ExistingEnumMember {
            name: "Value2".to_string(),
            comment: None,
        },
    );
    let mut existing = HashMap::new();
    existing.insert("OrderStatus".to_string(), members);

    let snapshot = build_model_enum_plan_json_with_existing(&open_api_object, Some(&existing))
        .expect("build enum plan fail");
    assert!(snapshot.contains("\"name\": \"Success\""));
}

#[test]
fn build_model_enum_plan_json_with_existing_should_not_preserve_auto_generated_name() {
    let open_api_object = OpenAPIObject::from_str(
        r#"
{
  "openapi": "3.1.0",
  "info": { "title": "enum-test", "version": "1.0.0" },
  "paths": {},
  "components": {
    "schemas": {
      "OrderStatus": { "type": "string", "enum": ["failed", "success"] }
    }
  }
}
"#,
    )
    .expect("build openapi object fail");

    let mut members = HashMap::new();
    // Historical auto-generated name from a previous order; should not be reused.
    members.insert(
        "success".to_string(),
        ExistingEnumMember {
            name: "Value1".to_string(),
            comment: None,
        },
    );
    let mut existing = HashMap::new();
    existing.insert("OrderStatus".to_string(), members);

    let snapshot = build_model_enum_plan_json_with_existing(&open_api_object, Some(&existing))
        .expect("build enum plan fail");
    assert!(snapshot.contains("\"value\": \"success\""));
    assert!(snapshot.contains("\"name\": \"Value2\""));
}

fn get_temp_test_dir(prefix: &str) -> PathBuf {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("get epoch fail")
        .as_millis();
    std::env::temp_dir().join(format!("{prefix}-{millis}"))
}

#[test]
fn generate_model_files_with_existing_should_preserve_translated_name() {
    let open_api_object = OpenAPIObject::from_str(
        r#"
{
  "openapi": "3.1.0",
  "info": { "title": "enum-test", "version": "1.0.0" },
  "paths": {},
  "components": {
    "schemas": {
      "OrderStatus": {
        "type": "string",
        "enum": ["成功", "失败"]
      }
    }
  }
}
"#,
    )
    .expect("build openapi object fail");

    // Simulate existing translated enums from previous generation
    let mut members = HashMap::new();
    members.insert(
        "成功".to_string(),
        ExistingEnumMember {
            name: "Success".to_string(),
            comment: None,
        },
    );
    members.insert(
        "失败".to_string(),
        ExistingEnumMember {
            name: "Failed".to_string(),
            comment: None,
        },
    );
    let mut existing = HashMap::new();
    existing.insert("OrderStatus".to_string(), members);

    let files = generate_model_files_with_existing(
        &open_api_object,
        ModelRenderStyle::Module,
        &["OrderStatus".to_string()],
        &existing,
    )
    .expect("generate model files with existing fail");

    let enum_file = files
        .get("OrderStatus.ts")
        .expect("enum file should be generated");
    // Should preserve the translated names
    assert!(enum_file.contains("Success"));
    assert!(enum_file.contains("Failed"));
}

#[test]
fn generate_model_files_with_existing_should_add_new_members() {
    let open_api_object = OpenAPIObject::from_str(
        r#"
{
  "openapi": "3.1.0",
  "info": { "title": "enum-test", "version": "1.0.0" },
  "paths": {},
  "components": {
    "schemas": {
      "OrderStatus": {
        "type": "string",
        "enum": ["成功", "失败", "新状态"]
      }
    }
  }
}
"#,
    )
    .expect("build openapi object fail");

    // Existing: only 2 members
    let mut members = HashMap::new();
    members.insert(
        "成功".to_string(),
        ExistingEnumMember {
            name: "Success".to_string(),
            comment: None,
        },
    );
    members.insert(
        "失败".to_string(),
        ExistingEnumMember {
            name: "Failed".to_string(),
            comment: None,
        },
    );
    let mut existing = HashMap::new();
    existing.insert("OrderStatus".to_string(), members);

    let files = generate_model_files_with_existing(
        &open_api_object,
        ModelRenderStyle::Module,
        &["OrderStatus".to_string()],
        &existing,
    )
    .expect("generate model files with existing fail");

    let enum_file = files
        .get("OrderStatus.ts")
        .expect("enum file should be generated");
    // Should preserve translated names
    assert!(enum_file.contains("Success"));
    assert!(enum_file.contains("Failed"));
    // New member should have auto-generated name
    assert!(enum_file.contains("Value3") || enum_file.contains("新状态"));
}

#[test]
fn generate_model_files_with_existing_should_preserve_comments() {
    let open_api_object = OpenAPIObject::from_str(
        r#"
{
  "openapi": "3.1.0",
  "info": { "title": "enum-test", "version": "1.0.0" },
  "paths": {},
  "components": {
    "schemas": {
      "AssignmentStatus": {
        "type": "integer",
        "enum": [0, 1, 2]
      }
    }
  }
}
"#,
    )
    .expect("build openapi object fail");

    // Simulate existing translated enums with comments from previous generation
    let mut members = HashMap::new();
    members.insert(
        "0".to_string(),
        ExistingEnumMember {
            name: "Enabled".to_string(),
            comment: Some("启用".to_string()),
        },
    );
    members.insert(
        "1".to_string(),
        ExistingEnumMember {
            name: "Disabled".to_string(),
            comment: Some("禁用".to_string()),
        },
    );
    members.insert(
        "2".to_string(),
        ExistingEnumMember {
            name: "Banned".to_string(),
            comment: Some("封禁".to_string()),
        },
    );
    let mut existing = HashMap::new();
    existing.insert("AssignmentStatus".to_string(), members);

    let files = generate_model_files_with_existing(
        &open_api_object,
        ModelRenderStyle::Module,
        &["AssignmentStatus".to_string()],
        &existing,
    )
    .expect("generate model files with existing fail");

    let enum_file = files
        .get("AssignmentStatus.ts")
        .expect("enum file should be generated");
    // Should preserve the translated names
    assert!(enum_file.contains("Enabled"));
    assert!(enum_file.contains("Disabled"));
    assert!(enum_file.contains("Banned"));
    // Should preserve the comments
    assert!(enum_file.contains("/** 启用 */"));
    assert!(enum_file.contains("/** 禁用 */"));
    assert!(enum_file.contains("/** 封禁 */"));
}
