use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{MediaTypeObject, ResponseHeaders, response_lihk::ResponseLink};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseObject {
    pub description: String,

    pub headers: Option<HashMap<String, ResponseHeaders>>,

    pub content: Option<HashMap<String, MediaTypeObject>>,

    pub links: Option<HashMap<String, ResponseLink>>,
}
