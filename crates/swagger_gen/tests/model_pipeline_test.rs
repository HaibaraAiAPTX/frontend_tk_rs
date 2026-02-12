use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use swagger_gen::model_pipeline::{
    ModelRenderStyle, build_model_ir_snapshot_json, generate_model_files, parse_openapi_to_model_ir,
};
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

fn get_temp_test_dir(prefix: &str) -> PathBuf {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("get epoch fail")
        .as_millis();
    std::env::temp_dir().join(format!("{prefix}-{millis}"))
}

