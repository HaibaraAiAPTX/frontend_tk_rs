use swagger_tk::{
    getter::get_schema_by_name,
    model::{OpenAPIObject, SchemaEnum},
};

use super::reference_object_extension::ReferenceObjectExtension;

pub trait SchemaEnumExtension {
    fn get_ts_type(&self) -> String;

    fn can_be_null(&self, open_api: &OpenAPIObject) -> bool;

    fn is_enum(&self, open_api: &OpenAPIObject) -> bool;

    /// 根据类型判断枚举是不是原始枚举（原始枚举不生成新的枚举文件）
    fn is_raw_type_enum(&self, r#type: &str) -> bool;

    /// 如果是原始枚举，则获取类型的字符串
    fn get_raw_enum_type(&self) -> Result<String, String>;
}

impl SchemaEnumExtension for SchemaEnum {
    fn get_ts_type(&self) -> String {
        match self {
            SchemaEnum::Ref(schema) => schema.get_type_name(),
            SchemaEnum::Object(_schema) => "object".to_string(),
            SchemaEnum::String(_schema) => "string".to_string(),
            SchemaEnum::Integer(_schema) => "number".to_string(),
            SchemaEnum::Number(_schema) => "number".to_string(),
            SchemaEnum::Boolean(_schema) => "boolean".to_string(),
            SchemaEnum::Array(schema) => {
                let child_type = schema.items.as_ref().get_ts_type();
                format!("Array<{}>", child_type)
            }
        }
    }

    fn can_be_null(&self, open_api: &OpenAPIObject) -> bool {
        match self {
            SchemaEnum::Ref(v) => get_schema_by_name(open_api, &v.get_type_name())
                .is_some_and(|x| x.can_be_null(open_api)),
            SchemaEnum::Object(v) => v.nullable.unwrap_or_default(),
            SchemaEnum::String(v) => v.nullable.unwrap_or_default(),
            SchemaEnum::Integer(v) => v.nullable.unwrap_or_default(),
            SchemaEnum::Number(v) => v.nullable.unwrap_or_default(),
            SchemaEnum::Boolean(v) => v.nullable.unwrap_or_default(),
            SchemaEnum::Array(v) => v.nullable.unwrap_or_default(),
        }
    }

    fn is_enum(&self, open_api: &OpenAPIObject) -> bool {
        match self {
            SchemaEnum::String(v) => v.r#enum.is_some(),
            SchemaEnum::Integer(v) => v.r#enum.is_some(),
            SchemaEnum::Number(v) => v.r#enum.is_some(),
            SchemaEnum::Ref(v) => get_schema_by_name(open_api, &v.get_type_name())
                .is_some_and(|x| x.is_enum(open_api)),
            _ => false,
        }
    }

    fn is_raw_type_enum(&self, r#type: &str) -> bool {
        matches!(r#type, "string" | "number")
    }

    fn get_raw_enum_type(&self) -> Result<String, String> {
        fn process_enum<T: ToString>(values: &[T], add_quotes: bool) -> String {
            values
                .iter()
                .map(|v| {
                    if add_quotes {
                        format!("\"{}\"", v.to_string())
                    } else {
                        v.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" | ")
        }

        match self {
            SchemaEnum::String(v) => v
                .r#enum
                .as_ref()
                .map(|values| process_enum(values, true)) // 对 String 类型添加双引号
                .ok_or_else(|| "not found enum".to_string()),
            SchemaEnum::Integer(v) => v
                .r#enum
                .as_ref()
                .map(|values| process_enum(values, false)) // 对 Integer 类型不添加双引号
                .ok_or_else(|| "not found enum".to_string()),
            SchemaEnum::Number(v) => v
                .r#enum
                .as_ref()
                .map(|values| process_enum(values, false)) // 对 Number 类型不添加双引号
                .ok_or_else(|| "not found enum".to_string()),
            _ => Err("not found enum".to_string()),
        }
    }
}
