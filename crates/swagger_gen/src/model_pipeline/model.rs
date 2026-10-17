use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelIr {
    pub models: Vec<ModelNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEnumPlan {
    pub schema_version: String,
    pub enums: Vec<ModelEnumPlanItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEnumPlanItem {
    pub enum_name: String,
    pub description: Option<String>,
    pub source: String,
    pub members: Vec<ModelEnumPlanMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEnumPlanMember {
    pub name: String,
    pub value: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumPatchDocument {
    pub schema_version: String,
    pub patches: Vec<EnumPatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumPatch {
    pub enum_name: String,
    pub members: Vec<EnumPatchMember>,
    pub source: Option<String>,
    pub confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumPatchMember {
    pub value: String,
    pub suggested_name: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelNode {
    pub name: String,
    pub description: Option<String>,
    pub kind: ModelKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelKind {
    Interface { properties: Vec<ModelProperty> },
    Enum { members: Vec<ModelEnumMember> },
    Alias { target: ModelType, nullable: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProperty {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub nullable: bool,
    pub r#type: ModelType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEnumMember {
    pub name: String,
    pub value: ModelLiteral,
    pub comment: Option<String>,
}

/// Represents an existing enum member parsed from generated TypeScript files.
/// Used for preserving both names and comments during regeneration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExistingEnumMember {
    pub name: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IntegerFormat {
    #[default]
    Unknown,
    Int32,
    Int64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NumberFormat {
    #[default]
    Unknown,
    Float,
    Double,
    Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegerSpec {
    #[serde(default)]
    pub format: IntegerFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberSpec {
    #[serde(default)]
    pub format: NumberFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScalarType {
    String,
    Boolean,
    Integer(IntegerSpec),
    Number(NumberSpec),
}

/// 类型槽位：类型本体 + 该槽位自身是否可空。
///
/// 字典值、索引签名这类位置需要「嵌套类型独立携带 nullable」。`render_type` 已用独立参数
/// 表达可空，所以这里不引入 `Nullable` 包装变体，避免与既有调用约定重复。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullableType {
    pub r#type: Box<ModelType>,
    #[serde(default)]
    pub nullable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModelType {
    Scalar(ScalarType),
    String,
    Number,
    Boolean,
    /// 无键值信息的自由对象：`additionalProperties: true` 或完全未声明。
    /// 仍渲染为裸 `object` / `dict[str, Any]`，不得用来承载字典语义。
    Object,
    Ref {
        name: String,
    },
    Array {
        item: Box<ModelType>,
    },
    Union {
        variants: Vec<ModelType>,
    },
    Literal {
        value: ModelLiteral,
    },
    /// 字典：`type: object` + `additionalProperties: <schema>`，键恒为 `string`。
    /// TS 渲染为 `Record<string, V>`，Python 渲染为 `dict[str, V]`。
    Dictionary {
        value: NullableType,
    },
    /// 内联固定键对象（`properties` 非空）。
    ///
    /// `additional_properties` 为 `Some` 时表示 `properties` + `additionalProperties: <schema>`
    /// 的混合形态，TS 渲染为交叉类型 `{ ... } & Record<string, V>`。
    InlineObject {
        properties: Vec<ModelProperty>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        additional_properties: Option<NullableType>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModelLiteral {
    String {
        value: String,
    },
    Integer {
        value: String,
    },
    Number {
        value: String,
        #[serde(default)]
        format: NumberFormat,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelRenderStyle {
    Declaration,
    Module,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnumConflictPolicy {
    OpenApiFirst,
    PatchFirst,
    ProviderFirst,
}

impl ModelRenderStyle {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "declaration" => Ok(Self::Declaration),
            "module" => Ok(Self::Module),
            _ => Err("`--style` expects declaration|module.".to_string()),
        }
    }
}

impl EnumConflictPolicy {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "openapi-first" => Ok(Self::OpenApiFirst),
            "patch-first" => Ok(Self::PatchFirst),
            "provider-first" => Ok(Self::ProviderFirst),
            _ => Err(
                "`--conflict-policy` expects openapi-first|patch-first|provider-first.".to_string(),
            ),
        }
    }
}
