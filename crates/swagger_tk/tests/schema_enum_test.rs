//! `SchemaEnum` 反序列化契约：`type` 缺失时的推断规则。
//!
//! OpenAPI 3.0 / 3.1 允许 schema 省略 `type`（`additionalProperties: {}` 就是合法写法）。
//! 历史上 `SchemaEnum::deserialize` 强制要求 `type`，导致一份 spec 只要出现
//! `{}` / 无 `type` 的 schema，整份 spec 都解析失败，且错误信息不指向具体位置。
//!
//! 这里固定「无 `type` 时」的推断规则，并确认既有显式 `type` 的分支没有被改动。
//! 另外钉住 `OpenAPIObject::from_str` 的**报错可定位性**（`frontend_tk_rs#2`）。

use std::str::FromStr;

use swagger_tk::model::{
    OpenAPIObject, OpenAPIParseError, SchemaEnum, SchemaObjectAdditionalProperties, SchemaTypeEnum,
};

fn parse(text: &str) -> OpenAPIObject {
    OpenAPIObject::from_str(text).unwrap_or_else(|err| panic!("parse fail: {err}"))
}

/// 整份 spec 解析失败时的报错文案。
fn parse_error(text: &str) -> String {
    parse_error_object(text).to_string()
}

/// 整份 spec 解析失败时的错误对象，用于断言 `path()` 等结构化信息。
fn parse_error_object(text: &str) -> OpenAPIParseError {
    OpenAPIObject::from_str(text).expect_err("spec should fail to parse")
}

/// 把一段 schema JSON 包进最小 spec 的 `components.schemas`。
fn schema_of(schema: &str) -> SchemaEnum {
    let open_api = parse(&format!(
        r#"{{"openapi":"3.1.0","info":{{"title":"t","version":"1.0.0"}},"paths":{{}},"components":{{"schemas":{{"A":{schema}}}}}}}"#
    ));
    open_api
        .components
        .as_ref()
        .and_then(|components| components.schemas.as_ref())
        .and_then(|schemas| schemas.get("A"))
        .expect("schema A should be parsed")
        .clone()
}

/// 直接反序列化单段 schema，用来固定错误路径的文案。
///
/// 不能走 [`schema_of`]：`OpenAPIObject::from_str` 是整体反序列化，schema 出错会让外壳
/// 一起失败，拿不到内层错误。
fn schema_error(schema: &str) -> String {
    serde_json::from_str::<SchemaEnum>(schema)
        .expect_err(&format!("schema {schema} should fail to deserialize"))
        .to_string()
}

/// 断言 schema 落到期望的 `SchemaEnum` 变体，且变体内记录的 `type` 与之一致。
fn assert_variant(schema: &SchemaEnum, expected: SchemaTypeEnum) {
    let actual = match schema {
        SchemaEnum::Ref(v) => panic!("expected {expected:?}, got Ref({v:?})"),
        SchemaEnum::Object(v) => v.r#type.clone(),
        SchemaEnum::String(v) => v.r#type.clone(),
        SchemaEnum::Integer(v) => v.r#type.clone(),
        SchemaEnum::Number(v) => v.r#type.clone(),
        SchemaEnum::Boolean(v) => v.r#type.clone(),
        SchemaEnum::Array(v) => v.r#type.clone(),
    };
    assert_eq!(actual, expected, "unexpected variant: {schema:?}");
    let variant_matches = matches!(
        (schema, &expected),
        (SchemaEnum::Object(_), SchemaTypeEnum::Object)
            | (SchemaEnum::String(_), SchemaTypeEnum::String)
            | (SchemaEnum::Integer(_), SchemaTypeEnum::Integer)
            | (SchemaEnum::Number(_), SchemaTypeEnum::Number)
            | (SchemaEnum::Boolean(_), SchemaTypeEnum::Boolean)
            | (SchemaEnum::Array(_), SchemaTypeEnum::Array)
    );
    assert!(variant_matches, "enum variant mismatch: {schema:?}");
}

/// issue #2 的三种失败形态必须都能解析。
#[test]
fn parses_additional_properties_empty_schema() {
    let open_api = parse(
        r#"{
          "openapi": "3.1.0",
          "info": { "title": "t", "version": "1.0.0" },
          "paths": {},
          "components": { "schemas": { "A": { "type": "object", "additionalProperties": {} } } }
        }"#,
    );
    let schema = open_api
        .components
        .expect("components")
        .schemas
        .expect("schemas")
        .remove("A")
        .expect("schema A");
    let SchemaEnum::Object(object) = schema else {
        panic!("A should be an object schema");
    };
    let value = object
        .additional_properties
        .expect("additionalProperties should be present");
    // `additionalProperties: {}` 语义等价于 `additionalProperties: true`（任意值）。
    let SchemaObjectAdditionalProperties::Schema(schema) = value else {
        panic!("expected an inferred schema, got {value:?}");
    };
    let SchemaEnum::Object(any) = &*schema else {
        panic!("`{{}}` should be inferred as object, got {schema:?}");
    };
    assert!(matches!(
        any.additional_properties,
        Some(SchemaObjectAdditionalProperties::Boolean(true))
    ));
}

#[test]
fn parses_typeless_schema_with_nullable() {
    let schema = schema_of(r#"{ "nullable": true }"#);
    let SchemaEnum::Object(object) = &schema else {
        panic!("typeless schema should be inferred as object, got {schema:?}");
    };
    assert_eq!(object.nullable, Some(true));
    assert!(object.properties.is_none());
}

#[test]
fn parses_typeless_schema_with_properties() {
    let schema = schema_of(r#"{ "properties": { "x": { "type": "string" } } }"#);
    let SchemaEnum::Object(object) = &schema else {
        panic!("typeless schema with properties should be inferred as object, got {schema:?}");
    };
    let properties = object.properties.as_ref().expect("properties");
    assert_eq!(properties.len(), 1);
    assert!(matches!(properties.get("x"), Some(SchemaEnum::String(_))));
}

#[test]
fn parses_empty_schema_as_free_form_object() {
    let schema = schema_of("{}");
    let SchemaEnum::Object(object) = &schema else {
        panic!("empty schema should be inferred as object, got {schema:?}");
    };
    assert!(object.properties.is_none());
    // 空 schema 等价于 `additionalProperties: true`，显式记录以免被读成封闭对象。
    assert!(matches!(
        object.additional_properties,
        Some(SchemaObjectAdditionalProperties::Boolean(true))
    ));
    // 描述为空是合法输入，不应触发 `description` 相关错误。
    assert!(schema.get_description().is_none());
}

/// 推断表的其它行：array / string enum / 数值 enum。
#[test]
fn infers_array_from_items() {
    let schema = schema_of(r#"{ "items": { "type": "string" } }"#);
    let SchemaEnum::Array(array) = &schema else {
        panic!("typeless schema with items should be inferred as array, got {schema:?}");
    };
    assert!(matches!(array.items.as_ref(), SchemaEnum::String(_)));
}

#[test]
fn infers_string_enum_from_enum_values() {
    let schema = schema_of(r#"{ "enum": ["a", "b"] }"#);
    assert_variant(&schema, SchemaTypeEnum::String);
    let SchemaEnum::String(string) = &schema else {
        unreachable!("checked above")
    };
    assert_eq!(
        string.r#enum.as_deref(),
        Some(&["a".to_string(), "b".to_string()][..])
    );
}

#[test]
fn infers_integer_enum_from_integer_values() {
    let schema = schema_of(r#"{ "enum": [1, 2] }"#);
    assert_variant(&schema, SchemaTypeEnum::Integer);
    let SchemaEnum::Integer(integer) = &schema else {
        unreachable!("checked above")
    };
    assert_eq!(integer.r#enum.as_deref(), Some(&[1, 2][..]));
}

#[test]
fn infers_number_enum_from_decimal_values() {
    let schema = schema_of(r#"{ "enum": [1.5, 2.5] }"#);
    assert_variant(&schema, SchemaTypeEnum::Number);
    let SchemaEnum::Number(number) = &schema else {
        unreachable!("checked above")
    };
    assert_eq!(number.r#enum.as_deref(), Some(&[1.5_f32, 2.5][..]));
}

/// 布尔 schema 带 `enum` 时必须明确报错。
///
/// `SchemaBool` 没有 `enum` 字段（只有 `SchemaString` / `SchemaInteger` / `SchemaNumber` 有），
/// 静默接受会让 `{"type":"boolean","enum":[true]}` 这样的约束凭空消失。
#[test]
fn rejects_boolean_schema_with_enum() {
    let err = schema_error(r#"{ "type": "boolean", "enum": [true] }"#);
    assert!(
        err.contains("`SchemaBool` has no `enum` field"),
        "got: {err}"
    );

    // 显式布尔类型 + `enum` 同样拦得住（上面那条只证明显式分支，这条证明带描述也不例外）。
    let err =
        schema_error(r#"{ "type": "boolean", "description": "开关", "enum": [true, false] }"#);
    assert!(
        err.contains("`SchemaBool` has no `enum` field"),
        "got: {err}"
    );

    // **推断**路径：无 `type` 时 `enum` 的第一个成员是布尔 → 推断成 boolean → 同样必须报错。
    // 这条才是「推断不会放过不可表达形态」的真正护栏；`{"enum":[true]}` 在真实 spec 里
    // 等价于「只允许 true 的布尔」，静默丢掉 `enum` 就是静默改变了契约。
    for json in [
        r#"{ "enum": [true] }"#,
        r#"{ "enum": [true, false] }"#,
        r#"{ "nullable": true, "enum": [false] }"#,
    ] {
        let err = schema_error(json);
        assert!(
            err.contains("`SchemaBool` has no `enum` field"),
            "{json} 应被拒绝，got: {err}"
        );
    }

    // 没有 `enum` 的布尔 schema 不受影响。
    let schema = schema_of(r#"{ "type": "boolean", "description": "开关" }"#);
    assert!(matches!(schema.get_description(), Some(description) if description == "开关"));

    // 无 `type` 且无 `enum` 的布尔形态无法从结构推断出来，只能靠显式 `type`——
    // 这也说明上面那组拒绝结论来自 `infer_enum_type`，不是别的线索。
    assert_variant(
        &schema_of(r#"{ "type": "boolean" }"#),
        SchemaTypeEnum::Boolean,
    );
}

/// 非法 `additionalProperties` schema 的报错必须**指向所属 schema**并带上具体原因。
///
/// 历史问题：`SchemaObjectAdditionalProperties` 用了 `#[serde(untagged)]`，serde 把输入缓冲成
/// `Content` 后逐个变体试，缓冲区不保留内层 `D::Error`，于是 `SchemaEnum` 写的诊断被整体替换成
/// `data did not match any variant of untagged enum……`，`line 2 column 61` 对单行压缩的
/// 真实 spec 毫无用处。
///
/// 现在两道保险都要在：
/// 1. `SchemaObjectAdditionalProperties` 手写 `Deserialize`（按 `Value` 分派），内层诊断不再被吞；
/// 2. `OpenAPIObject::from_str` 走 `serde_path_to_error`，报错带上 `components.schemas.<Name>`。
#[test]
fn locates_invalid_additional_properties_schema() {
    let err = parse_error(
        r#"{"openapi":"3.1.0","info":{"title":"t","version":"1.0.0"},"paths":{},
            "components":{"schemas":{"Order":{"type":"object","additionalProperties":{"type":["string","null"]}}}}}"#,
    );
    assert!(
        err.contains("components.schemas.Order"),
        "报错应指向所属 schema，got: {err}"
    );
    assert!(
        err.contains("neither a string nor absent"),
        "报错应带上内层诊断，got: {err}"
    );
    assert!(
        !err.contains("untagged enum SchemaObjectAdditionalProperties"),
        "缓冲层的默认文案不应再出现，got: {err}"
    );
}

/// 对照：同一个 schema 直接反序列化时文案完整——证明诊断本身写对了，丢的只是外层上下文。
#[test]
fn reports_full_diagnostic_when_schema_is_deserialized_directly() {
    let direct = schema_error(r#"{ "type": ["string", "null"] }"#);
    assert!(
        direct.contains("neither a string nor absent"),
        "got: {direct}"
    );
}

/// `OpenAPIParseError::path()` 是给消费方拼前缀用的结构化入口，必须与 `Display` 一致。
///
/// 语法错误的路径为空（它只有行列号，没有字段路径）。
#[test]
fn exposes_structured_error_path() {
    let err = parse_error_object(
        r#"{"openapi":"3.1.0","info":{"title":"t","version":"1.0.0"},"paths":{},
            "components":{"schemas":{"Order":{"type":"object","additionalProperties":{"type":["string","null"]}}}}}"#,
    );
    assert_eq!(err.path(), "components.schemas.Order");

    let syntax = parse_error_object(r#"{"openapi": "3.1.0", "info": }"#);
    assert_eq!(
        syntax.path(),
        "",
        "语法错误没有字段路径，got: {:?}",
        syntax.path()
    );
}

/// 非 schema 错误（纯语法错）仍要给出准确的行列号，不能被路径包装吃掉。
#[test]
fn keeps_position_for_syntax_errors() {
    let err = parse_error(r#"{"openapi": "3.1.0", "info": }"#);
    assert!(err.contains("line 1 column"), "got: {err}");
}

/// 截断的输入（`Category::Eof`）也要按语法错误报，不能挂半截路径。
///
/// `serde_path_to_error` 走到 `info` 才遇到 EOF，会留下 `info.?` 这种带 `?` 的路径。
/// 「文件没写完」的唯一有用坐标是原文行列号，所以这类错误必须与语法错误同样处理。
#[test]
fn reports_truncated_input_without_partial_path() {
    let err = parse_error(r#"{"openapi": "3.1.0", "info": {"title":"t""#);
    assert!(err.contains("EOF while parsing"), "got: {err}");
    assert!(
        !err.contains("info.?"),
        "截断输入不应挂半截路径，got: {err}"
    );
}

/// 文档末尾的多余内容必须继续被拒绝（`serde_json::from_str` 的既有行为）。
///
/// 手动驱动 `serde_json::Deserializer` 时 `from_str` 的「读完即结束」检查不会自动发生，
/// 少了 `end()` 就会把 `{...}{...}` 静默当成第一段解析成功。
#[test]
fn rejects_trailing_content_after_document() {
    let err = parse_error(
        r#"{"openapi":"3.1.0","info":{"title":"t","version":"1.0.0"},"paths":{}}{"x":1}"#,
    );
    assert!(err.contains("trailing characters"), "got: {err}");
}

/// `additionalProperties` 是非 bool / 非对象的 JSON 类型时，报错要说清「允许什么」。
///
/// 手写 `Deserialize` 之前，这类输入同样只剩 untagged 的默认文案。
///
/// 不含 `null`：`Option<SchemaObjectAdditionalProperties>` 把 `null` 读成 `None`，
/// 与「字段缺失」不可区分。这是 serde `Option` 的既有语义，本次不改。
#[test]
fn rejects_additional_properties_that_is_neither_bool_nor_schema() {
    for (json, kind) in [
        (r#""nope""#, "string"),
        (r#"3"#, "number"),
        (r#"[]"#, "array"),
    ] {
        let err = parse_error(&format!(
            r#"{{"openapi":"3.1.0","info":{{"title":"t","version":"1.0.0"}},"paths":{{}},
                "components":{{"schemas":{{"Order":{{"type":"object","additionalProperties":{json}}}}}}}}}"#
        ));
        assert!(
            err.contains("must be a boolean or a schema object"),
            "additionalProperties={json} 应被明确拒绝，got: {err}"
        );
        assert!(err.contains(kind), "报错应点明实际类型 {kind}，got: {err}");
        assert!(
            err.contains("components.schemas.Order"),
            "报错应指向所属 schema，got: {err}"
        );
    }
}

/// `additionalProperties` 里的非法 schema 报错要同时给出**字段名**与**具体原因**。
///
/// `serde_path_to_error` 只能追到宿主结构体（`components.schemas.Order`），追不到字段，
/// 所以字段名由 `SchemaObjectAdditionalProperties` 自己补——这条测试钉住两侧都在。
#[test]
fn names_the_additional_properties_field_in_the_error() {
    let err = parse_error(
        r#"{"openapi":"3.1.0","info":{"title":"t","version":"1.0.0"},"paths":{},
            "components":{"schemas":{"Order":{"type":"object","additionalProperties":{"type":"boolean","enum":[true]}}}}}"#,
    );
    assert!(
        err.contains("components.schemas.Order"),
        "报错应指向所属 schema，got: {err}"
    );
    assert!(
        err.contains("`additionalProperties`"),
        "报错应点明字段名，got: {err}"
    );
    assert!(
        err.contains("`SchemaBool` has no `enum` field"),
        "报错应带上内层诊断，got: {err}"
    );
}

/// 嵌在 `properties` → `items` 深处的 `additionalProperties` 也要能定位到**所属 schema**。
///
/// 路径**停在 schema 名**（`components.schemas.Page`）而不是继续下沉到
/// `properties.rows.items`：`SchemaEnum::deserialize` 会把整段 schema 缓冲成 `Value`
/// 再分派，`serde_path_to_error` 的链到那里就断了。所属 schema 名 + 具体原因已经满足
/// 「可定位」的口径（见 `docs/llm/frontend_tk_rs_model_ir.md` §2.3.1）。
#[test]
fn locates_additional_properties_nested_in_items() {
    let err = parse_error(
        r#"{"openapi":"3.1.0","info":{"title":"t","version":"1.0.0"},"paths":{},
            "components":{"schemas":{"Page":{"type":"object","properties":{"rows":{"type":"array","items":{"type":"object","additionalProperties":{"type":["string","null"]}}}}}}}}"#,
    );
    assert!(
        err.contains("components.schemas.Page"),
        "报错应指向所属 schema，got: {err}"
    );
    assert!(
        err.contains("`additionalProperties`"),
        "报错应点明字段名，got: {err}"
    );
    assert!(
        err.contains("neither a string nor absent"),
        "报错应带上内层诊断，got: {err}"
    );
}

/// `properties` 里的 schema 报错同样要带所属 schema 名（回归：这条路径本来就可见）。
#[test]
fn locates_schema_errors_under_properties() {
    let err = parse_error(
        r#"{"openapi":"3.1.0","info":{"title":"t","version":"1.0.0"},"paths":{},
            "components":{"schemas":{"Page":{"type":"object","properties":{"rows":{"type":["string","null"]}}}}}}"#,
    );
    assert!(
        err.contains("components.schemas.Page"),
        "报错应指向所属 schema，got: {err}"
    );
    assert!(
        err.contains("neither a string nor absent"),
        "报错应带上内层诊断，got: {err}"
    );
}

/// 合法 spec 仍然正常解析（路径包装不得改变成功路径）。
#[test]
fn still_parses_valid_specs() {
    let open_api = parse(
        r#"{"openapi":"3.1.0","info":{"title":"t","version":"1.0.0"},"paths":{},
            "components":{"schemas":{"Order":{"type":"object","additionalProperties":{"type":"string"}}}}}"#,
    );
    let schemas = open_api
        .components
        .expect("components")
        .schemas
        .expect("schemas");
    let SchemaEnum::Object(order) = schemas.get("Order").expect("Order") else {
        panic!("Order should be an object schema");
    };
    let Some(SchemaObjectAdditionalProperties::Schema(value)) = &order.additional_properties else {
        panic!(
            "expected schema-valued additionalProperties, got {:?}",
            order.additional_properties
        );
    };
    assert!(matches!(**value, SchemaEnum::String(_)));
}

/// 显式 `type` 的分支不得受推断影响（回归护栏）。
#[test]
fn keeps_explicit_type_branches_unchanged() {
    for (json, expected) in [
        (r#"{ "type": "string" }"#, SchemaTypeEnum::String),
        (r#"{ "type": "object" }"#, SchemaTypeEnum::Object),
        (r#"{ "type": "integer" }"#, SchemaTypeEnum::Integer),
        (r#"{ "type": "number" }"#, SchemaTypeEnum::Number),
        (r#"{ "type": "boolean" }"#, SchemaTypeEnum::Boolean),
        (
            r#"{ "type": "array", "items": { "type": "string" } }"#,
            SchemaTypeEnum::Array,
        ),
    ] {
        assert_variant(&schema_of(json), expected);
    }
}

/// 显式 `type` 优先于推断线索（`type: string` + `items` 仍当 string）。
#[test]
fn explicit_type_wins_over_shape_hints() {
    let schema = schema_of(r#"{ "type": "string", "items": { "type": "string" } }"#);
    assert_variant(&schema, SchemaTypeEnum::String);
}

#[test]
fn keeps_ref_branch_unchanged() {
    let schema = schema_of(r##"{ "$ref": "#/components/schemas/Other" }"##);
    assert!(matches!(schema, SchemaEnum::Ref(_)), "{schema:?}");
}

/// `SchemaEnum::Boolean` 也应当能给出描述（`SchemaBool` 原先没有该字段）。
#[test]
fn reads_description_from_boolean_schema() {
    let schema = schema_of(r#"{ "type": "boolean", "description": "flag" }"#);
    assert_eq!(schema.get_description(), Some(&"flag".to_string()));
}

/// 无 `type` 但 `nullable` 的取值不应被丢弃。
#[test]
fn keeps_typeless_schema_nullable_flag() {
    let schema = schema_of(r#"{ "nullable": true, "description": "anything" }"#);
    assert_eq!(schema.get_description(), Some(&"anything".to_string()));
    let SchemaEnum::Object(object) = &schema else {
        panic!("expected object, got {schema:?}");
    };
    assert_eq!(object.nullable, Some(true));
}

/// 非字符串 `type`（3.1 联合数组 / 显式 `null`）必须给出可定位的明确错误。
///
/// 本 crate 的 `SchemaTypeEnum` 是单值枚举，`type: ["string","null"]` 无法无歧义地映射到
/// 单一变体。选择明确报错而不是「按线索猜」：猜出来的模型可能悄悄错，而报错把决定权交回
/// 调用方。`additionalProperties: {}` 这类**缺失** `type` 的写法仍然正常推断。
#[test]
fn rejects_type_that_is_not_a_string() {
    for json in [
        r#"{ "type": ["string", "null"] }"#,
        r#"{ "type": ["object", "null"], "properties": { "x": { "type": "string" } } }"#,
        r#"{ "type": null }"#,
    ] {
        let err = schema_error(json);
        let message = err
            .rsplit_once(" at line ")
            .map(|(message, _)| message)
            .unwrap_or(&err);
        assert!(
            message.contains("is neither a string nor absent"),
            "schema {json} should be rejected with a locatable error, got: {err}"
        );
        assert!(message.contains("type: [...]"), "got: {err}");
    }
}

/// array 线索与 object 线索同时命中时取 array。
#[test]
fn infers_array_from_other_array_hints() {
    // `SchemaArray` 不表达 `properties`，取 object 会把数组语义整个丢掉。
    assert_variant(
        &schema_of(
            r#"{ "items": { "type": "string" }, "properties": { "a": { "type": "string" } } }"#,
        ),
        SchemaTypeEnum::Array,
    );
}

/// 只有 `items` 是 array 的推断线索；`uniqueItems` / `minItems` / `maxItems` 都不是。
///
/// 这三个关键字是约束而不是结构：单独出现时若判成 array，唯一可能的结局是撞上
/// `SchemaArray` 必填的 `items`，抛出 `missing field \`items\`` —— 一个完全不提
/// `uniqueItems` / `minItems`、把调用方指向错误方向的错误，而且会让整份 spec
/// 反序列化失败。JSON Schema 里它们本来就只是断言，对非数组是无害空操作，
/// 因此落成自由 object 更接近原意。
#[test]
fn ignores_constraint_keywords_when_inferring() {
    for json in [
        r#"{ "uniqueItems": true }"#,
        r#"{ "minItems": 1 }"#,
        r#"{ "maxItems": 10 }"#,
        r#"{ "minItems": 1, "maxItems": 10 }"#,
        r#"{ "uniqueItems": true, "minItems": 1 }"#,
    ] {
        assert_variant(&schema_of(json), SchemaTypeEnum::Object);
    }

    // 与 `items` 共存时不影响结论。
    assert_variant(
        &schema_of(r#"{ "items": { "type": "string" }, "uniqueItems": true, "minItems": 1 }"#),
        SchemaTypeEnum::Array,
    );
    // 显式 `type` 始终最高优先级。
    assert_variant(
        &schema_of(r#"{ "type": "array", "items": { "type": "string" }, "minItems": 1 }"#),
        SchemaTypeEnum::Array,
    );
}

/// `enum` 没有可用成员时必须明确报错：本 crate 的标量变体存不下空枚举，也存不下 `null`。
///
/// 若在这里「推断成功」，失败会推迟到变体反序列化，抛出与推断无关的
/// `invalid type: null, expected a string`，调用方看不出真正原因。
#[test]
fn rejects_enum_without_usable_members() {
    let err = schema_error(r#"{ "enum": [] }"#);
    assert!(err.contains("`enum` is empty"), "got: {err}");

    for json in [r#"{ "enum": [null, null] }"#, r#"{ "enum": ["a", null] }"#] {
        let err = schema_error(json);
        assert!(
            err.contains("cannot hold `null` members"),
            "schema {json} got: {err}"
        );
    }
}

/// `enum` 混用多种 JSON 类型时必须报错并列出实际出现的类型，不得静默降级。
#[test]
fn rejects_enum_mixing_multiple_json_types() {
    let err = schema_error(r#"{ "enum": ["a", 1, "b"] }"#);
    assert!(err.contains("`type` is missing"), "got: {err}");
    assert!(err.contains("string / integer"), "got: {err}");
    // 交替出现的类型不应被重复列出（去重必须跨位置生效）。
    // serde 的错误后缀会回显整段 JSON，所以只看 ` at line ` 之前的正文。
    let message = err
        .rsplit_once(" at line ")
        .map(|(message, _)| message)
        .unwrap_or(&err);
    assert_eq!(message.matches("integer").count(), 1, "got: {err}");
}

/// `enum` 成员本身是 object / array 时无法映射到任何标量变体，必须给出明确错误。
#[test]
fn rejects_enum_with_structured_members() {
    for json in [r#"{ "enum": [{ "a": 1 }] }"#, r#"{ "enum": [[1]] }"#] {
        let err = schema_error(json);
        assert!(
            err.contains("is neither string / integer / number / boolean"),
            "schema {json} got: {err}"
        );
    }
}
