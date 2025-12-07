use std::{cell::RefCell, collections::HashMap};

use crate::{
    core::{ApiContext, JsApiContextHelper},
    gen_api_trait::GenApi,
    utils::format_ts_code,
};
use swagger_tk::model::OpenAPIObject;

pub struct AxiosJsGen<'a> {
    open_api: &'a OpenAPIObject,

    content_list: RefCell<Vec<String>>,

    output: HashMap<String, String>,
}

impl<'a> AxiosJsGen<'a> {
    pub fn new(open_api: &'a OpenAPIObject) -> Self {
        Self {
            open_api,
            content_list: Default::default(),
            output: Default::default(),
        }
    }
}

impl<'a> GenApi for AxiosJsGen<'a> {
    fn gen_name_content_map(&mut self) {
        let content = format!(
            r#"import axios from 'axios'

{}"#,
            self.content_list.borrow().join("\n")
        );

        self.output.insert("index.js".to_string(), content);
    }

    fn gen_api(&self, api_context: &ApiContext) -> Result<(), String> {
        let helper = JsApiContextHelper::new(api_context);
        let ApiContext {
            func_name, method, ..
        } = api_context;
        let parameters = helper.get_parameters_string(false).unwrap_or_default();
        let request_body = helper
            .get_request_config_data()
            .map(|v| format!("\ndata: {v},"))
            .unwrap_or_default();
        let request_params = helper
            .get_request_config_params()
            .map(|v| format!("\nparams: {v},"))
            .unwrap_or_default();
        let url = helper.get_url();
        let content = format!(
            r#"export function {func_name} ({parameters}) {{
return axios.request({{
    url: {url},method:"{method}",{request_params}{request_body}
}})
}}
        "#
        );
        let content = format_ts_code(&content).map_err(|v| format!("format js code error: {v}"))?;
        self.content_list.borrow_mut().push(content);
        Ok(())
    }

    fn clear(&mut self) {
        self.output.clear();
        self.content_list.borrow_mut().clear();
    }

    fn get_outputs(&self) -> &HashMap<String, String> {
        &self.output
    }

    fn get_open_api(&self) -> &OpenAPIObject {
        &self.open_api
    }
}
