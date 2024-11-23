use inflector::cases::pascalcase::to_pascal_case;
use regex::Regex;

use crate::model::{
    OpenAPIObject, OperationObject, OperationObjectParameters, OperationObjectRequestBody,
    ParameterObjectIn, PathItemObject, ResponsesValue,
};

#[derive(Debug)]
pub struct ApiContext<'a> {
    /// 接口名称
    pub func_name: String,
    /// 接口URL
    pub url: &'a str,
    /// 接口方法
    pub method: &'a str,
    /// 原始数据
    pub path_item: &'a PathItemObject,
    /// 具体接口定义的原始数据
    pub operation: &'a OperationObject,
    /// 函数接口参数列表
    /// 该数据来源于params及requestBody
    pub func_parameters: Option<Vec<FuncParameter>>,
    /// 返回值类型
    pub response_type: Option<String>,
    /// 请求参数列表  
    /// @example  
    /// ```js
    /// axios.get("/api/test", {
    ///   params: {
    ///     data: value1,
    ///     key2: obj.value || true
    ///   }
    /// })
    /// ```
    pub query_params_list: Option<Vec<AttributeData>>,
    /// 路径参数列表  
    /// /api/user/{username}  
    /// /api/user/${data.username}
    pub path_params_list: Option<Vec<AttributeData>>,
    pub cookie_params_list: Option<Vec<AttributeData>>,
    pub header_params_list: Option<Vec<AttributeData>>,
    /// 请求体名称
    /// @example
    /// ```js
    /// axios.post("/api/test", {
    ///   data: // <request_body_name>
    /// })
    /// ```
    pub request_body_name: Option<String>,
    /// 请求体参数列表
    /// @example
    /// ```js
    /// axios.post("/api/test", {
    ///   data: {
    ///     data1: value1,
    ///     data2: value2
    ///   }
    /// })
    /// ```
    pub request_body_list: Option<Vec<AttributeData>>,
}

#[derive(Debug, Clone)]
pub struct FuncParameter {
    /// 参数名称
    pub name: String,
    /// 参数类型
    pub r#type: String,
    /// 是否可空
    /// 根据可空不可空，决定连接符是“:”还是“?:”  
    /// 根据是否可空对函数参数进行排序
    pub required: bool,
    /// 默认值
    /// 根据是否有默认值，决定后面是否跟上“= \<default\>”
    pub default: Option<String>,
    /// 在请求参数中是否过滤
    pub r#in: Option<ParameterObjectIn>,
}

#[derive(Debug)]
pub struct AttributeData {
    /// 参数名称
    pub name: String,

    /// 获取值表达式
    pub value_expression: String,
}

impl<'a> ApiContext<'a> {
    pub fn new(
        open_api_object: &OpenAPIObject,
        url: &'a str,
        method: &'a str,
        path_item: &'a PathItemObject,
        operation: &'a OperationObject,
    ) -> Self {
        let mut result = Self {
            url,
            method,
            path_item,
            operation,
            func_name: get_func_name(url, method, operation),
            func_parameters: None,
            response_type: get_raw_response_type(operation),
            query_params_list: None,
            path_params_list: None,
            cookie_params_list: None,
            header_params_list: None,
            request_body_name: None,
            request_body_list: None,
        };
        result.init_data(open_api_object);
        result
    }

    fn init_data(&mut self, _open_api_object: &OpenAPIObject) {
        self.init_ajax_data();
    }

    fn init_ajax_data(&mut self) {
        let mut params_data = get_func_parameters_object(self.operation);
        params_data.ensure_name_not_empty();
        match (params_data.parameters, params_data.request_body) {
            // 没有 请求参数 及 请求体
            (None, None) => {}
            // 没有 请求体 有 请求参数
            (Some(params), None) => {
                self.init_params(&params);
                self.func_parameters = Some(params);
            }
            // 有 请求体 没有 请求参数
            (None, Some(request_body)) => {
                self.request_body_name = Some(request_body.name.clone());
                self.func_parameters = Some(vec![request_body]);
            }
            // 有 请求体 也有 请求参数
            (Some(params), Some(request_body)) => {
                self.init_params(&params);
                self.request_body_name = Some(request_body.name.clone());
                self.func_parameters = Some([params, vec![request_body]].concat());
            }
        }
    }

    /// 初始化请求的 params
    fn init_params(&mut self, params: &Vec<FuncParameter>) {
        params.iter().for_each(|x| {
            if let Some(v) = &x.r#in {
                let value = AttributeData {
                    name: x.name.clone(),
                    value_expression: x.name.clone(),
                };
                self.add_param_to_list(v, value);
            }
        });
    }

    /// 将参数添加到相应的列表
    fn add_param_to_list(&mut self, param_type: &ParameterObjectIn, value: AttributeData) {
        match param_type {
            ParameterObjectIn::Query => {
                self.query_params_list
                    .get_or_insert_with(Vec::new)
                    .push(value);
            }
            ParameterObjectIn::Header => {
                self.header_params_list
                    .get_or_insert_with(Vec::new)
                    .push(value);
            }
            ParameterObjectIn::Path => {
                self.path_params_list
                    .get_or_insert_with(Vec::new)
                    .push(value);
            }
            ParameterObjectIn::Cookie => {
                self.cookie_params_list
                    .get_or_insert_with(Vec::new)
                    .push(value);
            }
        }
    }
}

/// 获取函数名称
fn get_func_name(url: &str, method: &str, operation: &OperationObject) -> String {
    if let Some(operation_id) = &operation.operation_id {
        return to_pascal_case(operation_id);
    } else {
        let reg = Regex::new(r"\{([^}]*)\}").unwrap();
        let mut name_list = url
            .split("/")
            .map(|s| {
                if s.contains("{") || s.contains("}") {
                    to_pascal_case(&reg.captures(s).unwrap()[1])
                } else {
                    to_pascal_case(s)
                }
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>();
        name_list.insert(0, to_pascal_case(method));

        let name_list = remove_repeat_words(&name_list);
        to_pascal_case(&name_list.join(" "))
    }
}

/// 根据列表，移除后一项与前一项的重复部分
fn remove_repeat_words(list: &[String]) -> Vec<String> {
    let mut result = Vec::<String>::new();
    let empty = String::new();
    for word in list {
        let last = result.last().unwrap_or(&empty);
        if word.starts_with(last) {
            result.push(word.replace(last, ""));
        } else {
            result.push(word.to_string());
        }
    }
    result
}

/// 获取函数参数列表
fn get_func_parameters_object(operation: &OperationObject) -> FuncParameterObject {
    let parameters = operation.parameters.as_ref().and_then(|v| {
        Some(
            v.iter()
                .map(|p| match p {
                    OperationObjectParameters::Parameter(p) => FuncParameter {
                        name: p.name.clone(),
                        r#type: p.schema.as_ref().unwrap().get_ts_type(),
                        required: p.required.unwrap_or(false),
                        default: None,
                        r#in: Some(p.r#in.clone()),
                    },
                    OperationObjectParameters::Reference(r) => FuncParameter {
                        name: String::new(),
                        r#type: r.get_type_name(),
                        required: true,
                        default: None,
                        r#in: None,
                    },
                })
                .collect::<Vec<FuncParameter>>(),
        )
    });

    let request_body = operation.request_body.as_ref().map_or(None, |x| match x {
        OperationObjectRequestBody::RequestBody(v) => {
            let data = v.content.iter().take(1).next();
            data.and_then(|(_, media_type)| {
                Some(FuncParameter {
                    name: String::new(),
                    r#type: media_type.schema.get_ts_type(),
                    required: v.required.unwrap_or(false),
                    default: None,
                    r#in: None,
                })
            })
        }
        OperationObjectRequestBody::Reference(v) => Some(FuncParameter {
            name: String::new(),
            r#type: v.get_type_name(),
            required: true,
            default: None,
            r#in: None,
        }),
    });

    FuncParameterObject {
        parameters,
        request_body,
    }
}

#[derive(Debug)]
pub struct FuncParameterObject {
    parameters: Option<Vec<FuncParameter>>,
    request_body: Option<FuncParameter>,
}

impl FuncParameterObject {
    /// 确保参数名称不为空
    fn ensure_name_not_empty(&mut self) {
        let mut sum: u16 = 1;
        if let Some(v) = &mut self.parameters {
            for param in v {
                if param.name.is_empty() {
                    param.name = format!("param{}", sum);
                    sum += 1;
                }
            }
        }
        if let Some(v) = &mut self.request_body {
            if v.name.is_empty() {
                v.name = format!("data{}", sum);
            }
        }
    }
}

fn get_raw_response_type(operation: &OperationObject) -> Option<String> {
    operation.responses.as_ref().and_then(|v| {
        v.get("200")
            .or_else(|| v.get("default"))
            .and_then(|d| match d {
                ResponsesValue::Response(v) => v
                    .content
                    .as_ref()
                    .and_then(|x| x.iter().next())
                    .map(|(_, data)| data.schema.get_ts_type()),
                ResponsesValue::Reference(v) => Some(v.get_type_name()),
            })
    })
}
