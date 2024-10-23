use serde::{Deserialize, Serialize};

use super::{ParameterObject, ReferenceObject};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum OperationObjectParameters {
    Parameter(ParameterObject),
    Reference(ReferenceObject),
}
