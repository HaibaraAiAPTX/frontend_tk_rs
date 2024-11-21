use crate::{
    gen_type::PropertyData,
    model::{
        OperationObjectParameters, ParameterObject, ReferenceObject, RequestBodyObject,
        ResponseObject, SchemaEnum,
    },
};

/// 从操作对象参数中获取属性数据
pub fn get_property_data_from_operation_object_parameters(
    data: &OperationObjectParameters,
) -> PropertyData {
    match data {
        OperationObjectParameters::Parameter(v) => get_property_data_from_parameter(v),
        OperationObjectParameters::Reference(v) => get_property_data_from_reference(v),
    }
}

/// 从参数对象中获取属性数据
pub fn get_property_data_from_parameter(data: &ParameterObject) -> PropertyData {
    let schema = data.schema.as_ref().expect("schema is none");
    let mut result = get_property_data_from_schema(schema);
    result.name = Some(data.name.clone());
    result.description = data.description.clone();
    result.required = data.required.clone();
    result.r#in = Some(data.r#in.clone());
    result.format = get_format_by_schema(schema);
    result
}

/// 从 schema 中获取属性数据
pub fn get_property_data_from_schema(data: &SchemaEnum) -> PropertyData {
    let r#type: Option<String> = get_type_from_schema(data);

    let mut result = PropertyData {
        name: None,
        description: None,
        required: None,
        r#type: r#type.clone(),
        format: None,
        children_type: None,
        r#in: None,
        r#enum: None,
        default: None,
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

/// 从引用对象中获取属性数据
pub fn get_property_data_from_reference(data: &ReferenceObject) -> PropertyData {
    PropertyData {
        name: None,
        description: None,
        required: None,
        r#type: get_interface_name_from_schema_name(&data.r#ref)
            .map_or(None, |v| Some(v.to_string())),
        format: None,
        children_type: None,
        r#in: None,
        default: None,
        r#enum: None,
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

/// 从请求体对象中获取属性数据
pub fn get_property_data_from_request_body_object(
    data: &RequestBodyObject,
) -> Option<PropertyData> {
    let content = data.content.values().next()?;
    Some(get_property_data_from_schema(&content.schema))
}

/// 从响应对象中获取属性数据
pub fn get_property_data_from_response_object(data: &ResponseObject) -> Option<PropertyData> {
    let content = data.content.as_ref().and_then(|v| v.values().next())?;
    Some(get_property_data_from_schema(&content.schema))
}

/// 从 schema 中获取格式
pub fn get_format_by_schema(data: &SchemaEnum) -> Option<String> {
    match data {
        SchemaEnum::String(v) => v
            .format
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or(String::from("unknown"))),
        SchemaEnum::Integer(v) => v.format.clone(),
        SchemaEnum::Number(v) => v.format.clone(),
        _ => None,
    }
}
