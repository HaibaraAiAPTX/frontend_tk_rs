use std::{collections::HashMap, vec};

use crate::{
    gen_type::PropertyData,
    model::{OperationObjectParameters, OperationObjectRequestBody, ResponsesValue},
};

use super::{
    get_property_data_from_operation_object_parameters, get_property_data_from_reference,
    get_property_data_from_request_body_object, get_property_data_from_response_object,
};

/// 从操作对象参数列表中获取属性数据列表
pub fn get_property_data_list_from_parameters(
    data: &Vec<OperationObjectParameters>,
) -> Vec<PropertyData> {
    data.iter()
        .map(|v| get_property_data_from_operation_object_parameters(v))
        .collect::<Vec<PropertyData>>()
}

/// 从请求体中获取属性数据列表
pub fn get_property_data_from_request_body(
    data: &OperationObjectRequestBody,
) -> Option<PropertyData> {
    match data {
        OperationObjectRequestBody::RequestBody(data) => get_property_data_from_request_body_object(data),
        OperationObjectRequestBody::Reference(data) => Some(get_property_data_from_reference(data)),
    }
}

/// 从众多的返回模型中返回正确时的属性数据列表
pub fn get_correct_property_data_list_from_responses(
    data: Option<&HashMap<String, ResponsesValue>>,
) -> Option<Vec<PropertyData>> {
    match data.and_then(|data| data.get("200").or_else(|| data.get("default")))? {
        ResponsesValue::Response(obj) => {
            get_property_data_from_response_object(obj).map(|v| vec![v])
        }
        ResponsesValue::Reference(v) => Some(vec![get_property_data_from_reference(v)]),
    }
}
