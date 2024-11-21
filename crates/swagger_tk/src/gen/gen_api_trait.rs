use std::collections::HashMap;

use crate::{gen::js_helper::ApiContext, model::OpenAPIObject};

pub trait GenApi {
    fn gen_apis(&mut self, data: &OpenAPIObject) -> Result<HashMap<String, String>, String> {
        let paths = data.paths.as_ref().ok_or("paths not found".to_string())?;

        for (url, path_item) in paths.iter() {
            for &(method, operation) in &[
                ("get", &path_item.get),
                ("put", &path_item.put),
                ("post", &path_item.post),
                ("delete", &path_item.delete),
            ] {
                if let Some(operation) = operation {
                    let api_context = ApiContext::new(data, url, method, path_item, operation);
                    let rt = self.gen_api(&api_context);
                    if rt.is_err() {
                        return Err(rt.unwrap_err());
                    }
                }
            }
        }

        Ok(self.gen_name_content_map())
    }

    fn gen_name_content_map(&mut self) -> HashMap<String, String>;

    fn gen_api(&mut self, api_context: &ApiContext) -> Result<(), String>;
}
