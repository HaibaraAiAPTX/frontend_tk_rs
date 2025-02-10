use crate::gen::{format_ts_code, js_helper::ApiContext, GenApi, JsApiContextHelper};
use inflector::cases::pascalcase::to_pascal_case;
use std::collections::HashMap;

#[derive(Default)]
pub struct AxiosTsGen {
    controller_apis_map: HashMap<String, Vec<String>>,
}

impl AxiosTsGen {
    fn gen_code(&mut self, api_context: &ApiContext) -> Result<String, String> {
        let helper = JsApiContextHelper::new(api_context);

        // 初始化方法参数request_data
        let parameters = helper.get_parameters_string().unwrap_or_default();

        // 获取基础URL，不包含查询参数
        let base_url = helper.get_url();

        // 请求体
        let request_data = helper.get_request_config_data();

        // 查询参数
        let request_params = helper.get_request_config_params();

        let other_params = match (request_data, request_params) {
            (None, None) => String::new(),
            (None, Some(v)) => format!(", {}", v),
            (Some(v), None) => format!(", {}", v),
            (Some(v1), Some(v2)) => format!(r#", undefined, {{ params: {v2}, data: {v1} }}"#),
        };

        // 返回类型
        let response_type = helper.get_response_type();

        let func_name = &api_context.func_name;
        let method = &api_context.method;

        let api_fun = format!(
            r#"{func_name}({parameters}) {{
  return this.{method}<{response_type}>({base_url}{other_params});
}}"#
        );
        Ok(api_fun)
    }
}

impl GenApi for AxiosTsGen {
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
import {{ BaseService }} from "./BaseService";

@singleton()
export class {}Service extends BaseService {{
{}
}}
"#,
                to_pascal_case(controller),
                apis.join("\n\n")
            );
            let content = format_ts_code(&content).unwrap();
            v.insert(to_pascal_case(controller), content);
        }
        v
    }

    fn clear(&mut self) {
        self.controller_apis_map.clear();
    }
}
