use serde::{Deserialize, Serialize};

use super::{ParameterObject, ReferenceObject};

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PathItemParameters {
    Parameter(ParameterObject),
    Reference(ReferenceObject),
}
