use swagger_tk::{
    getter::get_controller_description,
    model::OpenAPIObject,
};
use inflector::cases::pascalcase::to_pascal_case;
use std::collections::HashMap;
use crate::{built_in_api_trait::GenApi, core::{ApiContext, JsApiContextHelper}, utils::format_ts_code};

#[derive(Default)]
pub struct AxiosTsGen<'a> {
    controller_apis_map: HashMap<String, Vec<String>>,
    open_api: Option<&'a OpenAPIObject>,
    pub outputs: HashMap<String, String>,
}

impl<'a> AxiosTsGen<'a> {
    fn gen_code(&mut self, api_context: &ApiContext) -> Result<String, String> {
        let helper = JsApiContextHelper::new(api_context);

        // 初始化方法参数request_data
        let parameters = helper.get_parameters_string(true).unwrap_or_default();

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
        let description = api_context
            .description
            .map(|v| format!("/** {} */\n", v))
            .unwrap_or_default();

        let api_fun = format!(
            r#"
            {description}{func_name}({parameters}) {{
  return this.{method}<{response_type}>({base_url}{other_params});
}}"#
        );
        Ok(api_fun)
    }
}

impl<'a> GenApi<'a> for AxiosTsGen<'a> {
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

    fn gen_name_content_map(&mut self) {
        for (controller, apis) in self.controller_apis_map.iter() {
            let description = if let Some(open_api) = self.open_api {
                get_controller_description(open_api, controller)
            } else {
                None
            }
            .map(|s| format!("\n/** {} */", s))
            .unwrap_or_else(|| Default::default());

            let content = format!(
                r#"import {{ singleton }} from "tsyringe";
import {{ BaseService }} from "./BaseService";

{description}
@singleton()
export class {controller}Service extends BaseService {{
{}
}}
"#,
                apis.join("\n\n")
            );
            let content = format_ts_code(&content).unwrap();
            self.outputs.insert(format!("{}Service.ts", to_pascal_case(controller)), content);
        }
    }

    fn clear(&mut self) {
        self.controller_apis_map.clear();
        self.outputs.clear();
    }

    fn set_open_api(&mut self, open_api: &'a OpenAPIObject) {
        self.open_api = Some(open_api);
    }

    fn get_outputs(&self) -> &HashMap<String, String> {
        &self.outputs
    }
}
