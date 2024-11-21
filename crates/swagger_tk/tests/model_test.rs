use swagger_tk::model::OpenAPIObject;
use utils::get_open_api_text;

mod utils;

#[test]
fn model_test() {
    let text = get_open_api_text(None).unwrap();
    let open_api = OpenAPIObject::from_str(&text);
    assert!(open_api.is_ok(), "{}", open_api.unwrap_err());
}
