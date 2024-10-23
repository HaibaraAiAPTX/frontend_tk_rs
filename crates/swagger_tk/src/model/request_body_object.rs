use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::MediaTypeObject;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RequestBodyObject {
    pub description: Option<String>,

    pub content: HashMap<String, MediaTypeObject>,

    pub required: Option<bool>,
}
