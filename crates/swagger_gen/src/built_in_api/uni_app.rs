use crate::{
    built_in_api_trait::GenApi,
    core::{ApiContext, JsApiContextHelper},
    utils::format_ts_code,
};
use inflector::cases::pascalcase::to_pascal_case;
use std::{cell::RefCell, collections::HashMap};
use swagger_tk::model::OpenAPIObject;

pub struct UniAppGen<'a> {
    controller_apis_map: RefCell<HashMap<String, Vec<String>>>,

    /// 是否需要引入qs处理库
    need_import_qs: RefCell<bool>,

    open_api: &'a OpenAPIObject,

    outputs: HashMap<String, String>,
}

impl<'a> UniAppGen<'a> {
    pub fn new(open_api: &'a OpenAPIObject) -> Self {
        Self {
            open_api,
            controller_apis_map: Default::default(),
            need_import_qs: RefCell::new(false),
            outputs: Default::default(),
        }
    }

    fn gen_code(&self, api_context: &ApiContext) -> Result<String, String> {
        let helper = JsApiContextHelper::new(api_context);

        // 初始化方法参数request_data
        let parameters = helper.get_parameters_string(true).unwrap_or_default();

        // 请求的完整url及是否需要引入qs库
        let (url, import_qs) = helper.get_url_has_query();
        if import_qs {
            *self.need_import_qs.borrow_mut() = import_qs;
        }

        // 请求体
        let request_data = helper
            .get_request_config_data()
            .map(|s| format!(", {}", s))
            .unwrap_or_default();

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

impl<'a> GenApi for UniAppGen<'a> {
    fn gen_api(&self, api_context: &ApiContext) -> Result<(), String> {
        if let Some(tags) = &api_context.operation.tags {
            let api_fun = self.gen_code(api_context)?;

            for tag in tags.iter() {
                self.controller_apis_map
                    .borrow_mut()
                    .entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(api_fun.clone());
            }
        };
        Ok(())
    }

    fn gen_name_content_map(&mut self) {
        for (controller, apis) in self.controller_apis_map.borrow().iter() {
            let content = format!(
                r#"import {{ singleton }} from "tsyringe";
import {{ BaseService }} from "./BaseService";
{}

@singleton()
export class {}Service extends BaseService {{
{}
}}
"#,
                if *self.need_import_qs.borrow() {
                    "import qs from \"qs\";"
                } else {
                    ""
                },
                to_pascal_case(controller),
                apis.join("\n\n")
            );
            let content = format_ts_code(&content).unwrap();
            self.outputs
                .insert(format!("{}Service.ts", to_pascal_case(controller)), content);
        }
    }

    fn clear(&mut self) {
        *self.need_import_qs.borrow_mut() = false;
        self.controller_apis_map.borrow_mut().clear();
    }

    fn get_outputs(&self) -> &HashMap<String, String> {
        &self.outputs
    }
    
    fn get_open_api(&self) -> &OpenAPIObject {
        &self.open_api
    }
}
