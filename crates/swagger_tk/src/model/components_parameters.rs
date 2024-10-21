use super::{ParameterObject, ReferenceObject};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ComponentsParameters {
    Parameter(ParameterObject),
    Reference(ReferenceObject),
}
