use inflector::cases::pascalcase::to_pascal_case;
use crate::{gen::{format_ts_code, js_helper::ApiContext, GenApi, JsApiContextHelper}, model::OpenAPIObject};
use std::collections::HashMap;

#[derive(Default)]
pub struct UniAppGen<'a> {
    controller_apis_map: HashMap<String, Vec<String>>,

    /// 是否需要引入qs处理库
    need_import_qs: bool,

    open_api: Option<&'a OpenAPIObject>
}

impl<'a> UniAppGen<'a> {
    fn gen_code(&mut self, api_context: &ApiContext) -> Result<String, String> {
        let helper = JsApiContextHelper::new(api_context);

        // 初始化方法参数request_data
        let parameters = helper.get_parameters_string().unwrap_or_default();

        // 请求的完整url及是否需要引入qs库
        let (url, import_qs) = helper.get_url_has_query();
        if import_qs {
            self.need_import_qs = import_qs;
        }

        // 请求体
        let request_data = helper.get_request_config_data().map(|s| format!(", {}", s)).unwrap_or_default();

        // 返回类型
        let response_type = helper.get_response_type();

        let func_name = &api_context.func_name;
        let method = &api_context.method;

        let api_fun = format!(
            r#"{func_name}({parameters}) {{
  return this.{method}<{response_type}>({url}{request_data});
}}"#
        );
        Ok(api_fun)
    }
}

impl<'a> GenApi<'a> for UniAppGen<'a> {
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
{}

@singleton()
export class {}Service extends BaseService {{
{}
}}
"#,
                if self.need_import_qs {
                    "import qs from \"qs\";"
                } else {
                    ""
                },
                to_pascal_case(controller),
                apis.join("\n\n")
            );
            let content = format_ts_code(&content).unwrap();
            v.insert(to_pascal_case(controller), content);
        }
        v
    }
    
    fn clear(&mut self) {
        self.need_import_qs = false;
        self.controller_apis_map.clear();
    }

    fn set_open_api(&mut self, open_api: &'a OpenAPIObject) {
        self.open_api = Some(open_api);
    }
}
