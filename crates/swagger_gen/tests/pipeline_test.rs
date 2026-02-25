use swagger_gen::pipeline::{
    build_dry_run_plan, build_ir_snapshot_json, build_report_json, parse_openapi_to_ir,
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
