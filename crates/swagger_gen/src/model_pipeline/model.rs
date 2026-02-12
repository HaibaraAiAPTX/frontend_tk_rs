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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModelType {
    String,
    Number,
    Boolean,
    Object,
    Ref { name: String },
    Array { item: Box<ModelType> },
    Union { variants: Vec<ModelType> },
    Literal { value: ModelLiteral },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModelLiteral {
    String { value: String },
    Number { value: String },
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
