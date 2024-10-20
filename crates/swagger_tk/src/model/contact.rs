use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAPIInfoContact {
    pub name: Option<String>,

    pub url: Option<String>,
    
    pub email: Option<String>,
}
