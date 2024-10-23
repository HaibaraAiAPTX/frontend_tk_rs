use crate::{
    gen_type::PropertyData,
    model::{OperationObjectParameters, SchemaEnum},
};

pub fn get_property_data_list_from_parameters(
    data: &Option<Vec<OperationObjectParameters>>,
) -> Option<Vec<PropertyData>> {
    if let Some(data) = data {
        Some(
            data.iter()
                .map(|v| get_property_data_from_operation_object_parameters(v))
                .filter_map(|v| v)
                .collect::<Vec<PropertyData>>(),
        )
    } else {
        None
    }
}

pub fn get_property_data_from_operation_object_parameters(
    data: &OperationObjectParameters,
) -> Option<PropertyData> {
    match data {
        OperationObjectParameters::Parameter(v) => {
            let schema = v.schema.as_ref();
            let r#type = schema.map_or(None, |v| get_type_from_schema(v));
            Some(PropertyData {
                name: v.name.clone(),
                description: v.description.clone(),
                required: v.required,
                r#type: r#type.clone(),
                children_type: {
                    if r#type != Some("array".to_string()) {
                        return None;
                    }
                    if let Some(SchemaEnum::Array(array_data)) = schema {
                        let child_schema = array_data.items.as_ref();
                        let a: Option<String> = get_type_from_schema(child_schema);
                        Some(Box::new(PropertyData {
                            name: String::new(),
                            description: None,
                            required: None,
                            r#type: a,
                            children_type: None,
                        }))
                    } else {
                        None
                    }
                },
            })
        }
        OperationObjectParameters::Reference(v) => Some(PropertyData {
            name: String::new(),
            description: None,
            required: None,
            r#type: get_interface_name_from_schema_name(&v.r#ref)
                .map_or(None, |v| Some(v.to_string())),
            children_type: None,
        }),
    }
}

/// 根据 schema 的名称获取模型名称
pub fn get_interface_name_from_schema_name(name: &str) -> Option<&str> {
    let name_list = name.split("/");
    name_list.last()
}

/// 根据 schema 获取类型
pub fn get_type_from_schema(schema: &SchemaEnum) -> Option<String> {
    match schema {
        SchemaEnum::Ref(v) => {
            get_interface_name_from_schema_name(&v.r#ref).map_or(None, |v| Some(v.to_string()))
        }
        SchemaEnum::String(v) => Some(v.r#type.to_string()),
        SchemaEnum::Integer(v) => Some(v.r#type.to_string()),
        SchemaEnum::Number(v) => Some(v.r#type.to_string()),
        SchemaEnum::Boolean(v) => Some(v.r#type.to_string()),
        SchemaEnum::Array(v) => Some(v.r#type.to_string()),
        SchemaEnum::Object(v) => Some(v.r#type.to_string()),
    }
}
