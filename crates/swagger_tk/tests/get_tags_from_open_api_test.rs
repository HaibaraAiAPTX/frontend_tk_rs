use swagger_tk::getter::get_tags_from_open_api;
use utils::get_open_api_object;

mod utils;

#[test]
fn get_tags_from_open_api_test() {
    let open_api_object = get_open_api_object();
    let tags = get_tags_from_open_api(&open_api_object);
    assert!(tags.is_some());
}
