//! 字典类型（`additionalProperties`）渲染的验收快照。
//!
//! fixture 是从真实消费端 OpenAPI（`aptx-root/swagger/*.json`）裁剪出的 schema 子集，
//! 由 `scripts/extract-model-dictionary-fixture.mjs` 生成，因此这里的断言直接对应
//! 消费端期望的产物形态。

use std::str::FromStr;

use swagger_gen::model_pipeline::{ModelRenderStyle, generate_model_files};
use swagger_tk::model::OpenAPIObject;

const FIXTURE: &str = include_str!("fixtures/model_dictionary.json");

fn generate(style: ModelRenderStyle, names: &[&str]) -> std::collections::HashMap<String, String> {
    let open_api = OpenAPIObject::from_str(FIXTURE).expect("parse dictionary fixture fail");
    let only = names
        .iter()
        .map(|name| name.to_string())
        .collect::<Vec<_>>();
    generate_model_files(&open_api, style, &only).expect("generate model files fail")
}

fn render_one(name: &str) -> String {
    let mut files = generate(ModelRenderStyle::Module, &[name]);
    files
        .remove(&format!("{name}.ts"))
        .unwrap_or_else(|| panic!("{name}.ts should be generated"))
}

/// 样例 1：`{type:"object", additionalProperties:{type:"string", nullable:true}, nullable:true}`
#[test]
fn renders_nullable_string_dictionary() {
    let rendered = render_one("GuidStringDictionaryResultModel");

    assert!(
        rendered.contains("Data?: Record<string, string | null> | null"),
        "got:\n{rendered}"
    );
    assert!(
        !rendered.contains("Data?: object"),
        "dictionary must not degrade into bare object:\n{rendered}"
    );
}

/// 样例 2：required 且值全 string → 无 `?`、无 `| null`。
#[test]
fn renders_required_string_dictionary_without_nullability() {
    let rendered = render_one("MatchingQuestionDTO");

    assert!(
        rendered.contains("Data: Record<string, string>"),
        "got:\n{rendered}"
    );
    assert!(
        !rendered.contains("Data?: Record<string, string>"),
        "required dictionary must not be optional:\n{rendered}"
    );
}

/// 样例 3：值 number 且属性 nullable。
#[test]
fn renders_nullable_number_dictionary() {
    let rendered = render_one("AddPayrollRecordRequestModel");

    assert!(
        rendered.contains("SalaryItems?: Record<string, number> | null"),
        "got:\n{rendered}"
    );
}

/// 样例 4：值为 $ref 数组且可空 —— dprint 在 80 列处折成 leading-pipe 联合，
/// 语义等价，断言按真实格式化结果写。
#[test]
fn renders_nullable_ref_array_dictionary() {
    let rendered = render_one("StringClassScheduleListDTOListDictionaryResultModel");

    assert!(rendered.contains("Record<"), "got:\n{rendered}");
    assert!(
        rendered.contains("Array<import(\"./ClassScheduleListDTO\").ClassScheduleListDTO> | null"),
        "got:\n{rendered}"
    );
    // 值类型可空 + 属性可空：两个 `| null` 都必须存在
    assert!(
        rendered.matches("| null").count() >= 2,
        "value and property nullability must both be kept:\n{rendered}"
    );
    assert!(
        !rendered.contains("Data?: object"),
        "dictionary must not degrade into bare object:\n{rendered}"
    );
}

/// 样例 5：同一 schema 混合固定键对象与多个字典属性（PaperAnalysisDTO 模式）。
#[test]
fn renders_mixed_fixed_key_and_dictionary_properties() {
    let rendered = render_one("PaperAnalysisDTO");

    // 固定键对象必须内联展开，不能退化成 object
    assert!(
        rendered.contains("QuestionTypeNumbers?:"),
        "got:\n{rendered}"
    );
    assert!(rendered.contains("Essays?: number"), "got:\n{rendered}");
    assert!(
        rendered.contains("ReadingComprehension?: number"),
        "got:\n{rendered}"
    );
    assert!(rendered.contains("DifficultyNumbers?:"), "got:\n{rendered}");

    // 四个字典属性
    assert!(
        rendered.contains("KnowledgePointIncludeQuestions?: Record<string, Array<string>> | null"),
        "got:\n{rendered}"
    );
    assert!(
        rendered.contains("QuestionIncludeKnowledgePoints?: Record<string, Array<string>> | null"),
        "got:\n{rendered}"
    );
    assert!(
        rendered.contains("import(\"./KnowledgePointSimpleDTO\").KnowledgePointSimpleDTO"),
        "got:\n{rendered}"
    );
    assert!(
        rendered.contains("import(\"./PaperQuestionSimpleDTO\").PaperQuestionSimpleDTO"),
        "got:\n{rendered}"
    );

    assert!(
        !rendered.contains("object"),
        "no property may degrade into bare object:\n{rendered}"
    );
}

/// 值 number 的字典（market 的 StringDecimalDictionaryResultModel）。
#[test]
fn renders_number_dictionary_from_market_spec() {
    let rendered = render_one("StringDecimalDictionaryResultModel");

    assert!(
        rendered.contains("Data?: Record<string, number> | null"),
        "got:\n{rendered}"
    );
}

/// 回归：同一 fixture 内不含字典的 schema，产物必须与字典改造前一致。
#[test]
fn keeps_non_dictionary_schemas_unchanged() {
    let rendered = render_one("ClassScheduleListDTO");

    assert!(!rendered.contains("Record<"), "got:\n{rendered}");
    assert!(!rendered.contains("object"), "got:\n{rendered}");
    assert!(
        rendered.contains("export interface ClassScheduleListDTO"),
        "got:\n{rendered}"
    );
}

/// declaration 风格下字典同样正确（`model:enum-apply` 默认走 declaration）。
#[test]
fn renders_dictionary_in_declaration_style() {
    let files = generate(
        ModelRenderStyle::Declaration,
        &["GuidStringDictionaryResultModel"],
    );
    let rendered = files
        .get("GuidStringDictionaryResultModel.d.ts")
        .expect("declaration file should be generated");

    assert!(
        rendered.contains("declare interface GuidStringDictionaryResultModel"),
        "got:\n{rendered}"
    );
    assert!(
        rendered.contains("Data?: Record<string, string | null> | null"),
        "got:\n{rendered}"
    );
}

/// 顶层纯字典 schema 必须生成 `export type X = Record<...>`，不能是空 interface。
#[test]
fn renders_top_level_dictionary_schema_as_alias() {
    let fixture = r##"{
      "openapi": "3.1.0",
      "info": { "title": "top-level-dict", "version": "1.0.0" },
      "paths": {},
      "components": {
        "schemas": {
          "StringMap": { "type": "object", "additionalProperties": { "type": "string" } }
        }
      }
    }"##;
    let open_api = OpenAPIObject::from_str(fixture).expect("parse fail");
    let files = generate_model_files(&open_api, ModelRenderStyle::Module, &[]).expect("gen fail");
    let rendered = files
        .get("StringMap.ts")
        .expect("StringMap.ts should exist");

    assert!(
        rendered.contains("export type StringMap = Record<string, string>"),
        "got:\n{rendered}"
    );
    assert!(
        !rendered.contains("export interface StringMap"),
        "top-level dictionary must not be an empty interface:\n{rendered}"
    );
}
