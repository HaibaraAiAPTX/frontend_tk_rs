use swagger_tk::getter::get_paths_from_tag;
use utils::get_open_api_object;

mod utils;

#[test]
fn get_paths_from_tag_test() {
    let open_api_object = get_open_api_object();
    let paths = get_paths_from_tag(&open_api_object, "pet");
    assert!(paths.len() > 0, "没有找到控制器");
}
