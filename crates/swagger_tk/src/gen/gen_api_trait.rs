use std::collections::HashMap;

use crate::{gen::js_helper::ApiContext, model::OpenAPIObject};

pub trait GenApi<'a> {
    fn gen_apis(&mut self, data: &OpenAPIObject) -> Result<HashMap<String, String>, String> {
        self.clear();

        let paths = data.paths.as_ref().ok_or("paths not found".to_string())?;
        let mut paths_keys = paths.keys().collect::<Vec<_>>();
        paths_keys.sort();

        for url in paths_keys {
            let path_item = paths.get(url).ok_or(format!("can't find path data: {url}"))?;
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

    fn clear(&mut self);

    fn set_open_api(&mut self, open_api: &'a OpenAPIObject);
}
