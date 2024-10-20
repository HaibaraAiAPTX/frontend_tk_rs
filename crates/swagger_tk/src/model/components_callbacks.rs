use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use super::{CallbackValue, ReferenceObject};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ComponentsCallbacks {
    Callback(HashMap<String, CallbackValue>),
    Reference(ReferenceObject),
}
