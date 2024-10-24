use crate::{
    gen_type::PropertyData,
    model::{OperationObjectParameters, ParameterObject, ReferenceObject, SchemaEnum},
};

pub fn get_property_data_list_from_parameters(
    data: &Option<Vec<OperationObjectParameters>>,
) -> Option<Vec<PropertyData>> {
    if let Some(data) = data {
        Some(
            data.iter()
                .map(|v| get_property_data_from_operation_object_parameters(v))
                .collect::<Vec<PropertyData>>(),
        )
    } else {
        None
    }
}

pub fn get_property_data_from_operation_object_parameters(
    data: &OperationObjectParameters,
) -> PropertyData {
    match data {
        OperationObjectParameters::Parameter(v) => get_property_data_from_parameter(v),
        OperationObjectParameters::Reference(v) => get_property_data_from_reference(v),
    }
}

pub fn get_property_data_from_parameter(data: &ParameterObject) -> PropertyData {
    let schema = data.schema.as_ref().expect("schema is none");
    let mut result = get_property_data_from_schema(schema);
    result.name = Some(data.name.clone());
    result.description = data.description.clone();
    result.required = data.required.clone();
    result
}

pub fn get_property_data_from_schema(data: &SchemaEnum) -> PropertyData {
    let r#type = get_type_from_schema(data);

    let mut result = PropertyData {
        name: None,
        description: None,
        required: None,
        r#type: r#type.clone(),
        children_type: None,
    };

    if r#type == Some("array".to_string()) {
        if let SchemaEnum::Array(array_data) = data {
            result.children_type = Some(Box::new(get_property_data_from_schema(
                array_data.items.as_ref(),
            )))
        }
    }

    result
}

pub fn get_property_data_from_reference(data: &ReferenceObject) -> PropertyData {
    PropertyData {
        name: None,
        description: None,
        required: None,
        r#type: get_interface_name_from_schema_name(&data.r#ref)
            .map_or(None, |v| Some(v.to_string())),
        children_type: None,
    }
}

/// 根据 schema 的名称获取模型名称
pub fn get_interface_name_from_schema_name(name: &str) -> Option<&str> {
    let name_list = name.split("/");
    name_list.last()
}

/// 根据 schema 获取类型名称
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
