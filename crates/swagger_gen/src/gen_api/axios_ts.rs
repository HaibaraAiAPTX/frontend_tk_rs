use crate::{
    core::{ApiContext, JsApiContextHelper},
    gen_api_trait::GenApi,
    utils::format_ts_code,
};
use inflector::cases::pascalcase::to_pascal_case;
use std::{cell::RefCell, collections::HashMap};
use swagger_tk::{getter::get_controller_description, model::OpenAPIObject};

pub struct AxiosTsGen<'a> {
    controller_apis_map: RefCell<HashMap<String, Vec<String>>>,
    open_api: &'a OpenAPIObject,
    outputs: HashMap<String, String>,
}

impl<'a> AxiosTsGen<'a> {
    pub fn new(open_api: &'a OpenAPIObject) -> Self {
        Self {
            open_api,
            controller_apis_map: Default::default(),
            outputs: Default::default(),
        }
    }

    fn gen_code(&self, api_context: &ApiContext) -> Result<String, String> {
        let helper = JsApiContextHelper::new(api_context);

        // 初始化方法参数request_data
        let parameters = helper.get_parameters_string(true).unwrap_or_default();

        // 获取基础URL，不包含查询参数
        let url = helper.get_url();

        // 请求体
        let request_data = helper.get_request_config_data();

        // 查询参数
        let request_params = helper.get_request_config_params();

        // 返回类型
        let response_type = helper.get_response_type();

        let func_name = &api_context.func_name;
        let method = &api_context.method;
        let description = api_context
            .description
            .map(|v| format!("/** {} */\n", v))
            .unwrap_or_default();

        let other_params = {
            if ["post", "put", "patch"].contains(&api_context.method) {
                // 有第二个参数 data 的请求方法
                match (&request_data, &request_params) {
                    (None, None) => String::new(),
                    (None, Some(v)) => format!(r#", null, {{ params: {v} }}"#),
                    (Some(v), None) => format!(r#", {v}"#),
                    (Some(v1), Some(v2)) => format!(r#", {v1}, {{ params: {v2} }}"#),
                }
            } else {
                // 第二个参数是 config 的请求方法
                match (&request_data, &request_params) {
                    (None, None) => String::new(),
                    (None, Some(v)) => format!(r#", {{ params: {v} }}"#),
                    (Some(v), None) => format!(r#", {{ data: {v} }}"#),
                    (Some(v1), Some(v2)) => format!(r#", {{ data: {v1}, params: {v2} }}"#),
                }
            }
        };

        // 先确认基础部分
        let api_fun = format!(
            r#"
        {description}{func_name}({parameters}) {{
  return this.{method}<{response_type}>({url}{other_params});
}}"#
        );

        Ok(api_fun)
    }
}

impl<'a> GenApi for AxiosTsGen<'a> {
    fn gen_api(&self, api_context: &ApiContext) -> Result<(), String> {
        if let Some(tags) = &api_context.operation.tags {
            let api_fun = self.gen_code(api_context)?;

            for tag in tags.iter() {
                self.controller_apis_map
                    .borrow_mut()
                    .entry(tag.clone())
                    .or_default()
                    .push(api_fun.clone());
            }
        };
        Ok(())
    }

    fn gen_name_content_map(&mut self) {
        for (controller, apis) in self.controller_apis_map.borrow().iter() {
            let description = get_controller_description(self.open_api, controller)
                .map(|s| format!("\n/** {} */", s))
                .unwrap_or_default();

            let content = format!(
                r#"import {{ singleton }} from "tsyringe";
import {{ BaseService }} from "./BaseService";

{description}
@singleton()
export class {}Service extends BaseService {{
{}
}}
"#,
                to_pascal_case(controller),
                apis.join("\n\n")
            );
            let content = format_ts_code(&content).unwrap();
            self.outputs
                .insert(format!("{}Service.ts", to_pascal_case(controller)), content);
        }
    }

    fn clear(&mut self) {
        self.controller_apis_map.borrow_mut().clear();
        self.outputs.clear();
    }

    fn get_outputs(&self) -> &HashMap<String, String> {
        &self.outputs
    }

    fn get_open_api(&self) -> &OpenAPIObject {
        self.open_api
    }
}
