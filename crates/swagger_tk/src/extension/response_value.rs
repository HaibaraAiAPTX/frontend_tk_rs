use crate::model::{ResponseObject, ResponsesValue};

impl ResponsesValue {
    pub fn as_response(&self) -> Option<&ResponseObject> {
        match self {
            ResponsesValue::Response(response_object) => Some(response_object),
            _ => None,
        }
    }
}
