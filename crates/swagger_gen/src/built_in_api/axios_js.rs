use std::collections::HashMap;

use crate::{
    built_in_api_trait::GenApi,
    core::{ApiContext, JsApiContextHelper},
};
use swagger_tk::model::OpenAPIObject;

#[derive(Default)]
pub struct AxiosJsGen<'a> {
    open_api: Option<&'a OpenAPIObject>,

    content_list: Vec<String>,

    output: HashMap<String, String>,
}

impl<'a> GenApi<'a> for AxiosJsGen<'a> {
    fn gen_name_content_map(&mut self) {
        let content = format!(
            r#"
        import axios from 'axios'

        {}"#,
            self.content_list.join("\n\n")
        );

        self.output.insert("index.js".to_string(), content);
    }

    fn gen_api(&mut self, api_context: &ApiContext) -> Result<(), String> {
        let helper = JsApiContextHelper::new(api_context);
        let ApiContext {
            func_name, method, ..
        } = api_context;
        let parameters = helper.get_parameters_string(false).unwrap_or_default();
        let request_body = helper.get_request_config_data().map(|v| format!("\ndata: {v},")).unwrap_or_default();
        let request_params = helper.get_request_config_params().map(|v| format!("\nparams: {v},")).unwrap_or_default();
        let url = helper.get_url();
        let content = format!(
            r#"export function {func_name} ({parameters}) {{
            return axios.request({{
                url: {url},method:"{method}",{request_params}{request_body}
            }})
}}
        "#
        );
        self.content_list.push(content);
        Ok(())
    }

    fn clear(&mut self) {
        self.output.clear();
        self.content_list.clear();
    }

    fn set_open_api(&mut self, open_api: &'a OpenAPIObject) {
        self.open_api = Some(open_api)
    }

    fn get_outputs(&self) -> &HashMap<String, String> {
        &self.output
    }
}
