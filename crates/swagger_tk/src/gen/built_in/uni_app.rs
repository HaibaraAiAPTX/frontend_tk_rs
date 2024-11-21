use inflector::cases::pascalcase::to_pascal_case;

use crate::gen::{format_ts_code, js_helper::ApiContext, GenApi};
use std::collections::HashMap;

#[derive(Default)]
pub struct UniAppGen {
    controller_apis_map: HashMap<String, Vec<String>>,

    /// 是否需要引入qs处理库
    need_import_qs: bool,
}

impl UniAppGen {
    fn gen_code(&mut self, api_context: &ApiContext) -> Result<String, String> {
        // 初始化方法参数
        let parameters = api_context
            .func_parameters
            .as_ref()
            .and_then(|v| {
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
            .unwrap_or_default();

        let mut is_format = false;
        // 初始化 path 变量
        let mut url = {
            if let Some(v) = &api_context.path_params_list {
                is_format = true;
                let mut url = format!("{}", api_context.url);
                v.iter().for_each(|v| {
                    url = url.replace(
                        &format!("{{{}}}", v.name),
                        &format!("${{{}}}", v.value_expression),
                    );
                });
                url
            } else {
                format!("{}", api_context.url)
            }
        };

        // 初始化 query search
        if let Some(v) = &api_context.query_params_list {
            if !v.is_empty() {
                let mut data = String::from("{");
                self.need_import_qs = true;
                for i in v {
                    if i.name == i.value_expression {
                        data += &format!("{},", i.name);
                    } else {
                        data += &format!("{}:{},", i.name, i.value_expression);
                    }
                }
                let data = format!("{}}}", &data[..data.len() - 1]);
                url += &format!("?${{qs.stringify({})}}", data);
                is_format = true;
            }
        }

        // 初始化请求 data
        let request_data = match (
            &api_context.request_body_name,
            &api_context.request_body_list,
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

        let api_fun = format!(
            r#"{}({}) {{
  return this.{}<{}>({}{});
}}"#,
            api_context.func_name,
            parameters,
            api_context.method,
            api_context
                .response_type
                .as_ref()
                .unwrap_or(&"void".to_string()),
            if is_format {
                format!("`{}`", url)
            } else {
                format!("\"{}\"", url)
            },
            if let Some(v) = request_data {
                format!(",{}", v)
            } else {
                "".to_string()
            }
        );
        Ok(api_fun)
    }
}

impl GenApi for UniAppGen {
    fn gen_api(&mut self, api_context: &ApiContext) -> Result<(), String> {
        if let Some(tags) = &api_context.operation.tags {
            let api_fun = self.gen_code(api_context)?;

            for tag in tags.iter() {
                self.controller_apis_map
                    .entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(api_fun.clone());
            }
        };
        Ok(())
    }

    fn gen_name_content_map(&mut self) -> HashMap<String, String> {
        let mut v: HashMap<String, String> = HashMap::<String, String>::new();
        for (controller, apis) in self.controller_apis_map.iter() {
            let content = format!(
                r#"import {{ singleton }} from "tsyringe";
import {{ BaseService }} from "./BaseService";{}

@singleton()
export class {}Service extends BaseService {{
{}
}}
"#,
                if self.need_import_qs {
                    "\nimport qs from \"qs\";"
                } else {
                    ""
                },
                to_pascal_case(controller),
                apis.join("\n\n")
            );
            let content = format_ts_code(&content).unwrap();
            v.insert(controller.to_string(), content);
        }
        v
    }
}
