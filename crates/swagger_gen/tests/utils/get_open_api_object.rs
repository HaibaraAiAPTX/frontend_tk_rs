use swagger_tk::model::OpenAPIObject;

use super::get_open_api_text;

#[allow(dead_code)]
pub fn get_open_api_object(filename: Option<&str>) -> OpenAPIObject {
    let text = get_open_api_text(filename).unwrap();
    let object = OpenAPIObject::from_str(&text).unwrap();
    object
}
