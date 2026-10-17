use serde::{Deserialize, Serialize};

use super::SchemaEnum;

/// `additionalProperties` 的取值：`bool` 或一段 schema。
///
/// **这里是本 crate 唯一会吞掉 schema 报错的位置**，排查时先看这一段：
///
/// - `#[serde(untagged)]` 会把输入先缓冲成 `Content`，缓冲区**不保留内层 `D::Error`**，
///   于是 `SchemaEnum::deserialize` 里那些精心写的诊断文案（「非字符串 `type`」
///   「`SchemaBool` 没有 `enum` 字段」等）走到这一层会被整体替换成
///   `data did not match any variant of untagged enum SchemaObjectAdditionalProperties`。
/// - 手写 `Deserialize` 也救不回来：`deserialize` 只能返回 `D::Error`，而 `D::Error`
///   可能是借用型的 `&str`（`serde_json::Error` / `serde::de::value::Error` 不是），
///   无法安全地把内层错误取出来改写，所以「透传内层文案」在这个位置做不到。
/// - 影响面仅限 Model IR 这条路径。走 `components.schemas`（`HashMap<String, SchemaEnum>`）
///   或 API IR / `get_ts_type()` 时 schema 是直接反序列化的，不经过本枚举，文案正常可见。
///   见 `tests/schema_enum_test.rs::documents_error_text_loss_on_buffered_path`。
///
/// 因此：**报错本身仍然会让整份 spec 解析失败，只是文案丢细节**。要真正可定位（带
/// `components.schemas.X` 路径），需要改成「先按 `serde_json::Value` 解析、再逐节点手动
/// 反序列化并要求 `D::Error: Display`」，那是跨 crate 的接口改动，未在本 issue 内做。
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum SchemaObjectAdditionalProperties {
    Boolean(bool),
    Schema(Box<SchemaEnum>),
}
