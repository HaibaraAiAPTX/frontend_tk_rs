use serde::{Deserialize, Serialize};

use super::{ParameterObject, ReferenceObject};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum PathItemParameters {
    Parameter(ParameterObject),
    Reference(ReferenceObject),
}
