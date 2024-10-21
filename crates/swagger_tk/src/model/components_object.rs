use super::SchemaEnum;
use super::{
    ComponentsCallbacks, ComponentsHeaders, ComponentsParameters, ComponentsPathItems,
    ComponentsResponses,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct ComponentsObject {
    pub schemas: Option<HashMap<String, SchemaEnum>>,

    pub responses: Option<HashMap<String, ComponentsResponses>>,

    pub parameters: Option<HashMap<String, ComponentsParameters>>,

    pub headers: Option<HashMap<String, ComponentsHeaders>>,

    pub callbacks: Option<HashMap<String, ComponentsCallbacks>>,

    #[serde(rename = "pathItems")]
    pub path_items: Option<HashMap<String, ComponentsPathItems>>,
}
