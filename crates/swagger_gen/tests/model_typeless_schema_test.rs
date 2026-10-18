//! 「无 `type` 的 schema」在模型管线里的契约验收。
//!
//! 背景：`additionalProperties: {}` 这类**没有 `type`** 的 schema 是合法 OpenAPI，
//! 但 `SchemaEnum` 原先强制要求 `type`，导致 `SchemaObjectAdditionalProperties`
//! 的 untagged 分支全部失配、**整份 spec 反序列化失败**。
//! （该枚举现已改为手写 `Deserialize`，报错会带 `components.schemas.<Name>` 路径与具体原因，
//! 见 `crates/swagger_tk/src/model/schema/schema_object_additional_properties.rs`。）
//!
//! `swagger_tk` 侧的推断规则见 `crates/swagger_tk/src/model/schema/schema_enum.rs`；
//! 本文件锁住的是**推断结果进入模型 IR / 产物**后的形态，防止下游把「任意值」读成
//! 「封闭对象」或把字典退化成裸 `object`。

use std::str::FromStr;

use swagger_gen::model_pipeline::{
    ModelRenderStyle, build_model_ir_snapshot_json, generate_model_files,
};
use swagger_tk::model::OpenAPIObject;

/// 把若干具名 schema 包成一份最小可解析 spec，返回 IR 快照 JSON 与产物。
fn generate(schemas: &str) -> (String, std::collections::HashMap<String, String>) {
    // 用占位符替换而不是 `format!`：spec 里全是 `{}`，转义括号极易写错。
    let fixture = r#"{
      "openapi": "3.1.0",
      "info": { "title": "typeless", "version": "1.0.0" },
      "paths": {},
      "components": { "schemas": __SCHEMAS__ }
    }"#
    .replace("__SCHEMAS__", schemas);
    let open_api = OpenAPIObject::from_str(&fixture)
        .unwrap_or_else(|err| panic!("typeless fixture must parse, got: {err}\n{fixture}"));
    let ir = build_model_ir_snapshot_json(&open_api).expect("model ir fail");
    let files =
        generate_model_files(&open_api, ModelRenderStyle::Module, &[]).expect("generate fail");
    (ir, files)
}

fn render(schemas: &str, name: &str) -> String {
    let (_ir, mut files) = generate(schemas);
    files
        .remove(&format!("{name}.ts"))
        .unwrap_or_else(|| panic!("{name}.ts should be generated, got {:?}", files.keys()))
}

/// 缺陷复现：`additionalProperties: {}` 不再让整份 spec 反序列化失败。
#[test]
fn parses_spec_with_empty_additional_properties() {
    let (ir, files) = generate(r#"{ "AnyMap": { "type": "object", "additionalProperties": {} } }"#);

    assert!(ir.contains("AnyMap"), "got:\n{ir}");
    assert!(files.contains_key("AnyMap.ts"), "got {:?}", files.keys());
}

/// 无 `type` 且带 `properties` 的 schema 仍展开为具名接口，而不是裸 `object`。
#[test]
fn renders_typeless_schema_with_properties_as_inline_object() {
    let rendered = render(
        r#"{ "Typeless": { "properties": {
              "Name": { "type": "string", "description": "名称" },
              "Count": { "type": "integer" }
           }, "required": ["Name"] } }"#,
        "Typeless",
    );

    assert!(
        rendered.contains("export interface Typeless"),
        "got:\n{rendered}"
    );
    assert!(rendered.contains("Name: string"), "got:\n{rendered}");
    assert!(rendered.contains("Count?: number"), "got:\n{rendered}");
    assert!(
        !rendered.contains(": object"),
        "properties must not degrade into bare object:\n{rendered}"
    );
}

/// 无 `type` 且带 `items` 的 schema 推断为数组。
#[test]
fn renders_typeless_schema_with_items_as_array() {
    let rendered = render(
        r#"{ "TypelessList": { "items": { "type": "string" } } }"#,
        "TypelessList",
    );

    assert!(
        rendered.contains("export type TypelessList = Array<string>"),
        "got:\n{rendered}"
    );
}

/// 无 `type` 且带 `enum` 的 schema 按成员推断为字面量联合。
#[test]
fn renders_typeless_schema_with_enum_as_literal_union() {
    let rendered = render(
        r#"{ "Typeless": { "properties": { "Level": { "enum": ["a", "b"] } } } }"#,
        "Typeless",
    );

    assert!(
        rendered.contains(r#"Level?: "a" | "b""#),
        "got:\n{rendered}"
    );
}

/// `additionalProperties: {}` 是「任意值」而不是「封闭对象」：
/// 值类型落成自由 `object`，不能退化成空 interface，也不能被当成 `additionalProperties: false`。
#[test]
fn keeps_empty_additional_properties_as_free_object_value() {
    let rendered = render(
        r#"{ "AnyMap": { "type": "object", "additionalProperties": {} } }"#,
        "AnyMap",
    );

    // 顶层纯字典 ⇒ 类型别名；值类型是自由 object。
    assert!(
        rendered.contains("export type AnyMap = Record<string, object>"),
        "got:\n{rendered}"
    );
    assert!(
        !rendered.contains("export interface AnyMap"),
        "empty `additionalProperties` must stay a dictionary, not a closed interface:\n{rendered}"
    );
}

/// 无 `type` 的 `nullable` 属性：可空标记必须保留（`object | null`）。
#[test]
fn keeps_nullable_on_typeless_schema() {
    let rendered = render(
        r#"{ "Typeless": { "properties": {
              "Payload": { "nullable": true, "description": "任意负载" }
           } } }"#,
        "Typeless",
    );

    assert!(
        rendered.contains("Payload?: object | null"),
        "nullable must survive type inference:\n{rendered}"
    );
    assert!(
        rendered.contains("/** 任意负载 */"),
        "description must survive type inference:\n{rendered}"
    );
}

/// 完全空 schema `{}` 的顶层：等价于 `additionalProperties: true`，产物是自由 object 别名。
#[test]
fn renders_empty_schema_as_free_object_alias() {
    let rendered = render(r#"{ "Anything": {} }"#, "Anything");

    assert!(
        rendered.contains("export type Anything = object"),
        "got:\n{rendered}"
    );
}

/// 布尔属性上的 `description` 必须出现在产物注释里（`SchemaBool` 原先没有该字段，
/// 与 `type: boolean` 的属性一起被静默丢掉）。
#[test]
fn renders_description_on_boolean_property() {
    let rendered = render(
        r#"{ "Typeless": { "properties": {
              "Enable": { "type": "boolean", "description": "启用标识" }
           } } }"#,
        "Typeless",
    );

    assert!(
        rendered.contains("/** 启用标识 */"),
        "boolean property description must not be dropped:\n{rendered}"
    );
    assert!(rendered.contains("Enable?: boolean"), "got:\n{rendered}");
}
