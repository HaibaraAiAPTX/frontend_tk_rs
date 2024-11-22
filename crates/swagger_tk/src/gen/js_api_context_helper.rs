use super::js_helper::ApiContext;

pub struct JsApiContextHelper<'a> {
    api_context: &'a ApiContext<'a>,
}

impl<'a> JsApiContextHelper<'a> {
    pub fn new(context: &'a ApiContext) -> Self {
        Self {
            api_context: context,
        }
    }

    /// 初始化方法参数
    pub fn get_parameters_string(&self) -> Option<String> {
        self.api_context.func_parameters.as_ref().and_then(|v| {
            let mut data = v.clone();
            data.sort_by(|a, b| b.required.cmp(&a.required));
            Some(
                data.iter()
                    .map(|p| {
                        format!(
                            "{}{}{}",
                            p.name,
                            if p.required { ":" } else { "?:" },
                            p.r#type
                        )
                    })
                    .collect::<Vec<String>>()
                    .join(", "),
            )
        })
    }

    /// 获取 path 替换后的 url
    pub fn get_url(&self) -> String {
        if let Some(v) = &self.api_context.path_params_list {
            let mut url = format!("{}", self.api_context.url);
            v.iter().for_each(|v| {
                url = url.replace(
                    &format!("{{{}}}", v.name),
                    &format!("${{{}}}", v.value_expression),
                );
            });
            format!("`{}`", url)
        } else {
            format!("\"{}\"", self.api_context.url)
        }
    }

    /// 获取完整的请求 url，包含是否需要引入 qs 库
    ///
    /// 返回值：
    /// - `String`: 完整的请求 url，包含 query
    /// - `bool`: 是否需要引入 qs 库处理 params 对象
    pub fn get_url_has_query(&self) -> (String, bool) {
        let url: String = self.get_url();
        if let Some(v) = &self.api_context.query_params_list {
            if !v.is_empty() {
                let mut data = String::from("{");
                for i in v {
                    if i.name == i.value_expression {
                        data += &format!("{},", i.name);
                    } else {
                        data += &format!("{}:{},", i.name, i.value_expression);
                    }
                }
                let data = format!("{}}}", &data[..data.len() - 1]);
                let url = format!(
                    "`{}?${{qs.stringify({})}}`",
                    url[1..&url.len() - 1].to_string(),
                    data
                );
                (url, true)
            } else {
                (url, false)
            }
        } else {
            (url, false)
        }
    }

    /// 获取返回值的类型，如果没有就返回void
    pub fn get_response_type(&self) -> String {
        self.api_context
            .response_type
            .as_ref()
            .unwrap_or(&"void".to_string())
            .to_string()
    }

    /// 获取请求配置里面的请求体
    pub fn get_request_config_data(&self) -> String {
        // 初始化请求 data
        let request_data = match (
            &self.api_context.request_body_name,
            &self.api_context.request_body_list,
        ) {
            (None, None) => None,
            (None, Some(list)) => {
                if !list.is_empty() {
                    let mut result = String::from("{");
                    for v in list {
                        if v.name == v.value_expression {
                            result += &v.name;
                        } else {
                            result += &format!("{}:{}", v.name, v.value_expression);
                        }
                    }
                    Some(format!("{}}}", &result[..result.len() - 1]))
                } else {
                    None
                }
            }
            (Some(name), None) => Some(name.to_string()),
            (Some(_), Some(_)) => {
                panic!("这是不可能出现的情况，api_context的初始化出问题了");
            }
        };

        if let Some(v) = request_data {
            format!(",{}", v)
        } else {
            "".to_string()
        }
    }
}
