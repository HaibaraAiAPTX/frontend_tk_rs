use std::collections::HashMap;
use crate::gen::{js_helper::ApiContext, GenApi};

pub struct AxiosTs;

impl GenApi for AxiosTs {
    #[allow(unused_variables)]
    fn gen_api(&mut self, api_context: &ApiContext) -> Result<(), String> {
        todo!()
    }
    
    fn gen_name_content_map(&mut self) -> HashMap<String, String> {
        todo!()
    }
}