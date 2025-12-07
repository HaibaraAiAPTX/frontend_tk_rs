use serde::{Deserialize, Serialize};

use super::{PathItemObject, ReferenceObject};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAPIWebhooks {
    PathItem(Box<PathItemObject>),
    Reference(ReferenceObject),
}
