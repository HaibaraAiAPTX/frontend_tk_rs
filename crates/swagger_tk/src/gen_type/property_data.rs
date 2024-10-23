use serde::{Deserialize, Serialize};

/// 属性描述对象  
/// @example
/// ```json
/// {
///   "name": "name",
///   "description": "姓名",
///   "required": true,
///   "type": "string"
/// }
/// ```
/// ```json
/// {
///   "name": "age",
///   "description": "年龄",
///   "required": true,
///   "type": "number | string"
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyData {
    pub name: String,

    pub description: Option<String>,

    pub required: Option<bool>,

    /// 类型的字符串，如 Array
    /// 但是这个不一定有，因为 Enum 类型在部分语言中可以不指定类型，此时就为空
    pub r#type: Option<String>,

    /// 子类型，如 Array\<TestDTO\> 中的 TestDTO
    pub children_type: Option<Box<PropertyData>>
}
