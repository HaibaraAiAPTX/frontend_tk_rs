use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use swagger_gen::pipeline::{
    build_dry_run_plan, build_ir_snapshot_json, build_report_json, generate_functions_contract_v1,
    generate_react_query_contract_v1, generate_vue_query_contract_v1, parse_openapi_to_ir,
};
use utils::get_open_api_object;

mod utils;

#[test]
fn parse_openapi_to_ir_test() {
    let open_api_object = get_open_api_object(None);
    let ir = parse_openapi_to_ir(&open_api_object).expect("parse openapi to ir fail");
    assert!(!ir.endpoints.is_empty());
}

#[test]
fn build_dry_run_plan_test() {
    let open_api_object = get_open_api_object(None);
    let plan = build_dry_run_plan(&open_api_object).expect("build dry run plan fail");
    assert!(plan.endpoint_count > 0);
    assert!(!plan.transform_steps.is_empty());
    assert!(plan.metrics.total_ms >= plan.metrics.parse_ms);
}

#[test]
fn build_ir_snapshot_json_test() {
    let open_api_object = get_open_api_object(None);
    let snapshot = build_ir_snapshot_json(&open_api_object).expect("build ir snapshot fail");
    assert!(snapshot.contains("endpoints"));
}

#[test]
fn build_report_json_test() {
    let open_api_object = get_open_api_object(None);
    let report = build_report_json(&open_api_object).expect("build report json fail");
    assert!(report.contains("renderer_reports"));
    assert!(report.contains("metrics"));
}

#[test]
fn generate_functions_contract_v1_test() {
    let open_api_object = get_open_api_object(None);
    let output_root = get_temp_test_dir("pipeline-functions");

    let first = generate_functions_contract_v1(&open_api_object, &output_root)
        .expect("first functions generation fail");
    assert!(first.endpoint_count > 0);
    assert!(!first.planned_files.is_empty());

    let second = generate_functions_contract_v1(&open_api_object, &output_root)
        .expect("second functions generation fail");
    assert!(second.skipped_files > 0);

    fs::remove_dir_all(&output_root).expect("remove temp dir fail");
}

#[test]
fn generate_react_and_vue_query_contract_v1_test() {
    let open_api_object = get_open_api_object(None);
    let react_root = get_temp_test_dir("pipeline-react-query");
    let vue_root = get_temp_test_dir("pipeline-vue-query");

    let react_plan = generate_react_query_contract_v1(&open_api_object, &react_root)
        .expect("react query generation fail");
    let vue_plan = generate_vue_query_contract_v1(&open_api_object, &vue_root)
        .expect("vue query generation fail");

    assert!(react_plan.endpoint_count > 0);
    assert!(vue_plan.endpoint_count > 0);
    assert!(
        react_plan
            .planned_files
            .iter()
            .any(|f| f.path.ends_with(".query.ts"))
    );
    assert!(
        react_plan
            .planned_files
            .iter()
            .any(|f| f.path.ends_with(".mutation.ts"))
    );
    assert!(
        vue_plan
            .planned_files
            .iter()
            .any(|f| f.path.ends_with(".query.ts"))
    );
    assert!(
        vue_plan
            .planned_files
            .iter()
            .any(|f| f.path.ends_with(".mutation.ts"))
    );

    let react_query_file = react_plan
        .planned_files
        .iter()
        .find(|f| f.path.ends_with(".query.ts"))
        .expect("react query file not found");
    let react_query_content = fs::read_to_string(react_root.join(&react_query_file.path))
        .expect("read react query file fail");
    assert!(react_query_content.contains("signal: queryContext?.signal"));
    assert!(react_query_content.contains("__query: queryContext?.meta"));
    assert!(react_query_content.contains("normalizeInput"));

    fs::remove_dir_all(&react_root).expect("remove react temp dir fail");
    fs::remove_dir_all(&vue_root).expect("remove vue temp dir fail");
}

fn get_temp_test_dir(prefix: &str) -> PathBuf {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("get epoch fail")
        .as_millis();
    std::env::temp_dir().join(format!("{prefix}-{millis}"))
}
