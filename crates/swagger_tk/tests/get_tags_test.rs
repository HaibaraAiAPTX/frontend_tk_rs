use swagger_tk::getter::get_tags;
use utils::get_open_api_object;

mod utils;

#[test]
fn get_tags_test() {
  let open_api_object = get_open_api_object();
  let tags = get_tags(&open_api_object);
  assert!(tags.len() > 0);
}