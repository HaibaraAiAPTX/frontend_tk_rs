use serde::{Deserialize, Serialize};

use super::{ParameterObject, ReferenceObject};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OperationObjectParameters {
    Parameter(ParameterObject),
    Reference(ReferenceObject),
}
