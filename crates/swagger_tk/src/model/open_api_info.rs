use super::{OpenAPIInfoContact, OpenAPIInfoLicense};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAPIInfo {
    /// 标题
    pub title: String,

    /// 简介
    pub summary: Option<String>,

    /// 详细介绍
    pub description: Option<String>,

    #[serde(rename = "termsOfService")]
    pub terms_of_service: Option<String>,

    pub contact: Option<OpenAPIInfoContact>,

    pub license: Option<OpenAPIInfoLicense>,

    /// 版本号
    pub version: String,
}
