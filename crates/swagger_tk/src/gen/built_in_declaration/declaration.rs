use std::{collections::HashMap, fmt::Display};

use crate::{
    gen::format_ts_code,
    model::{OpenAPIObject, SchemaEnum, SchemaInteger, SchemaNumber, SchemaString},
};

pub struct TypescriptDeclarationGen<'a> {
    pub open_api: &'a OpenAPIObject,
}

impl<'a> TypescriptDeclarationGen<'a> {
    pub fn gen_declaration_by_name(&self, name: &str) -> Result<(String, bool), String> {
        let schema = self.get_schema(name)?;
        self.gen_declaration_by_schema(schema, name)
    }

    pub fn gen_declarations(&self) -> Result<HashMap<String, String>, String> {
        let schemas = self.get_schemas()?;
        let mut result = HashMap::<String, String>::new();
        for (name, schema) in schemas {
            let (content, is_enum) = self.gen_declaration_by_schema(schema, name)?;
            let new_name = if is_enum {
                format!("{name}.ts")
            } else {
                format!("{name}.d.ts")
            };
            result.insert(new_name, content);
        }
        Ok(result)
    }

    /// 生成声明
    fn gen_declaration_by_schema(&self, schema: &SchemaEnum, name: &str) -> Result<(String, bool), String> {
        match schema {
            SchemaEnum::Ref(v) => {
                let ref_name = v.r#ref.split("/").last().unwrap();
                return self.gen_declaration_by_name(ref_name);
            }
            SchemaEnum::Object(v) => {
                let description = v.description.as_ref().map(|s| s.as_str());
                let required = v.required.as_ref();
                let properties = v
                    .properties
                    .as_ref()
                    .ok_or_else(|| "没有找到object的属性值".to_string())?
                    .iter()
                    .map(|(k, v)| {
                        let symbol = required
                            .map(|v| v.contains(k))
                            .map(|v| if v { ":" } else { "?:" })
                            .unwrap_or("?:");
                        let mut r#type = {
                            if v.is_enum(&self.open_api) {
                                let file_name = v.get_ts_type();
                                format!("import(\"./{}\").{}", file_name, file_name)
                            } else {
                                v.get_ts_type()
                            }
                        };
                        if v.can_be_null(&self.open_api) {
                            r#type = format!("{} | null", r#type);
                        }
                        format!("{}{}{}", k, symbol, r#type)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let description = description
                    .map(|v| format!("/* {v} */"))
                    .unwrap_or_default();
                let result = format!(
                    r#"{description}
            declare interface {name} {{
                {properties}
    }}"#
                );

                Ok((format_ts_code(&result)?, false))
            }
            SchemaEnum::String(v) => {
                Ok((v.try_gen_enum(name)?, true))
            }
            SchemaEnum::Integer(v) => {
                Ok((v.try_gen_enum(name)?, true))
            }
            SchemaEnum::Number(v) => {
                Ok((v.try_gen_enum(name)?, true))
            }
            SchemaEnum::Boolean(_) => {
                return Err(format!("{} 不支持布尔类型", name));
            }
            SchemaEnum::Array(_) => {
                return Err(format!("{} 不支持数组类型", name));
            }
        }
    }

    /// 获取 schemas
    fn get_schemas(&self) -> Result<&HashMap<String, SchemaEnum>, String> {
        self.open_api
            .components
            .as_ref()
            .ok_or_else(|| "components is not found".to_string())?
            .schemas
            .as_ref()
            .ok_or_else(|| "schemas is not found".to_string())
    }

    /// 获取 schema
    fn get_schema(&self, name: &str) -> Result<&SchemaEnum, String> {
        let schemas = self.get_schemas()?;
        schemas.get(name).ok_or_else(|| format!("{} 未找到", name))
    }
}

trait GenEnum {
    fn try_gen_enum(&self, name: &str) -> Result<String, String>;
}

trait HasEnum {
    fn get_enum(&self) -> Option<&Vec<Self::EnumType>>;
    fn get_description(&self) -> Option<&str>;
    type EnumType: Display;
}

impl<T> GenEnum for T
where
    T: HasEnum,
{
    fn try_gen_enum(&self, name: &str) -> Result<String, String> {
        if self.get_enum().is_none() {
            return Err(format!("{} enum is not found", name));
        }

        let description = self.get_description();
        let enum_list = self.get_enum().unwrap();
        let mut count: u16 = 1;
        let enum_body = enum_list
            .iter()
            .map(|x| {
                let v = format!("Value{count} = {x}");
                count += 1;
                v
            })
            .collect::<Vec<_>>()
            .join(",\n");
        let description = {
            if let Some(v) = description {
                format!("/* {v} */")
            } else {
                String::new()
            }
        };
        let code = format!(
            r#"{description}
            export enum {name} {{
            {enum_body}
        }}"#
        );
        Ok(format_ts_code(&code)?)
    }
}

macro_rules! impl_has_enum {
    ($type:ty, $enum_type:ty) => {
        impl HasEnum for $type {
            fn get_enum(&self) -> Option<&Vec<Self::EnumType>> {
                self.r#enum.as_ref()
            }

            fn get_description(&self) -> Option<&str> {
                self.description.as_ref().map(|x| x.as_str())
            }

            type EnumType = $enum_type;
        }
    };
}

// 使用宏来实现 HasEnum trait
impl_has_enum!(SchemaString, String);
impl_has_enum!(SchemaInteger, i32);
impl_has_enum!(SchemaNumber, f32);
