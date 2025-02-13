use std::collections::HashMap;
use swagger_tk::model::OpenAPIObject;
use crate::core::ApiContext;

pub trait GenApi<'a> {
    /// 由外部调用，生成方法，调用后自动使用 gen_name_content_map 返回内容
    fn gen_apis(&mut self, data: &OpenAPIObject) -> Result<(), String> {
        self.clear();

        let paths = data.paths.as_ref().ok_or("paths not found".to_string())?;
        let mut paths_keys = paths.keys().collect::<Vec<_>>();
        paths_keys.sort();

        for url in paths_keys {
            let path_item = paths
                .get(url)
                .ok_or(format!("can't find path data: {url}"))?;
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
        self.gen_name_content_map();
        Ok(())
    }

    /// 获取 hashmap，key：文件名 value: 文件内容
    fn gen_name_content_map(&mut self);

    /// 各个生成器需要实现的生成单个 api 的实现
    fn gen_api(&mut self, api_context: &ApiContext) -> Result<(), String>;

    /// 清除生成器的缓存
    fn clear(&mut self);

    /// 设置 open_api 对象
    fn set_open_api(&mut self, open_api: &'a OpenAPIObject);

    /// 获取输出
    fn get_outputs(&self) -> &HashMap<String, String>;
}
