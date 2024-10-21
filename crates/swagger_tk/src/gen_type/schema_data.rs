use super::PropertyData;

/// 数据结构描述对象
pub struct SchemaData {
  /// 数据结构名称
  pub name: String,

  /// 数据结构描述
  pub description: Option<String>,

  /// 属性列表
  pub properties: Option<Vec<PropertyData>>,
}