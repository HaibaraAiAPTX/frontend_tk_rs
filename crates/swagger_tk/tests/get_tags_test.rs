use swagger_tk::getter::{get_tags, get_tags_from_open_api};
use utils::get_open_api_object;

mod utils;

#[test]
fn get_tags_test() {
    let open_api_object = get_open_api_object(None);
    let tags = get_tags(&open_api_object);
    assert!(tags.len() > 0);
}

#[test]
fn get_tags_from_open_api_test() {
    let open_api_object = get_open_api_object(None);
    let tags = get_tags_from_open_api(&open_api_object);
    assert!(tags.is_some());
}
