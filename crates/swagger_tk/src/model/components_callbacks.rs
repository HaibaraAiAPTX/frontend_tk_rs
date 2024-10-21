use std::collections::HashMap;

use super::{CallbackValue, ReferenceObject};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ComponentsCallbacks {
    Callback(HashMap<String, CallbackValue>),
    Reference(ReferenceObject),
}
