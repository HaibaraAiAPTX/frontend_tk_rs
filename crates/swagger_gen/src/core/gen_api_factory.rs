use std::{cell::RefCell, collections::HashMap};

use swagger_tk::model::OpenAPIObject;

use crate::built_in_api_trait::GenApi;

// 类型别名简化代码
pub type GenFactory = Box<dyn for<'a> Fn(&'a OpenAPIObject) -> Box<dyn GenApi + 'a>>;

// 注册表
#[derive(Default)]
pub struct GenRegistry {
    factories: RefCell<HashMap<String, GenFactory>>,
}

impl GenRegistry {
    pub fn register(&self, name: &str, factory: GenFactory) {
        self.factories.borrow_mut().insert(name.to_string(), factory);
    }

    pub fn create<'a>(&self, name: &str, open_api: &'a OpenAPIObject) -> Option<Box<dyn GenApi + 'a>> {
        self.factories.borrow().get(name).map(|f| f(open_api))
    }
}
