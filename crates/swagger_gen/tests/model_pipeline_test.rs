use std::{
    fs,
    path::PathBuf,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use swagger_gen::model_pipeline::{
    EnumConflictPolicy, EnumPatch, EnumPatchMember, ModelRenderStyle, build_model_enum_plan_json,
    build_model_ir_snapshot_json, generate_model_files, generate_model_files_with_enum_patch,
    parse_openapi_to_model_ir,
};
use swagger_tk::model::OpenAPIObject;
use utils::get_open_api_object;

mod utils;

#[test]
fn parse_openapi_to_model_ir_test() {
    let open_api_object = get_open_api_object(None);
    let ir = parse_openapi_to_model_ir(&open_api_object).expect("parse openapi to model ir fail");
    assert!(!ir.models.is_empty());
    assert!(ir.models.iter().any(|model| model.name == "Order"));
}

#[test]
fn build_model_ir_snapshot_json_test() {
    let open_api_object = get_open_api_object(None);
    let snapshot =
        build_model_ir_snapshot_json(&open_api_object).expect("build model ir snapshot fail");
    assert!(snapshot.contains("\"models\""));
    assert!(snapshot.contains("\"name\": \"Order\""));
}

#[test]
fn build_model_enum_plan_json_test() {
    let open_api_object = get_open_api_object(None);
    let snapshot = build_model_enum_plan_json(&open_api_object).expect("build enum plan fail");
    assert!(snapshot.contains("\"schema_version\": \"1\""));
    assert!(snapshot.contains("\"enums\""));
}

#[test]
fn generate_model_files_with_two_styles_test() {
    let open_api_object = get_open_api_object(None);
    let declaration_files =
        generate_model_files(&open_api_object, ModelRenderStyle::Declaration, &[])
            .expect("generate declaration files fail");
    assert!(declaration_files.contains_key("Order.d.ts"));

    let module_files = generate_model_files(&open_api_object, ModelRenderStyle::Module, &[])
        .expect("generate module files fail");
    let order_module = module_files
        .get("Order.ts")
        .expect("Order.ts should be generated");
    assert!(order_module.contains("export interface Order"));
}

#[test]
fn generate_model_files_with_name_filter_test() {
    let open_api_object = get_open_api_object(None);
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

fn get_temp_test_dir(prefix: &str) -> PathBuf {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("get epoch fail")
        .as_millis();
    std::env::temp_dir().join(format!("{prefix}-{millis}"))
}
