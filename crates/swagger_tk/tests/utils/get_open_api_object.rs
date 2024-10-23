use swagger_tk::model::OpenAPIObject;

use super::get_open_api_text;

pub fn get_open_api_object() -> OpenAPIObject {
    let text = get_open_api_text().unwrap();
    let object = OpenAPIObject::from_str(&text).unwrap();
    object
}
