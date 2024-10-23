use swagger_tk::getter::{get_tags_from_path_item, get_tags_from_paths};
use utils::get_open_api_object;

mod utils;

#[test]
fn tags_test() {
    let open_api_object = get_open_api_object();
    let paths = &open_api_object.paths;
    assert!(paths.is_some(), "paths 不能为空");
    let paths = paths.as_ref().unwrap();
    let path_item = paths.get("/pet");
    assert!(path_item.is_some(), "没有找到 api");
    let tags = get_tags_from_path_item(&path_item.unwrap());
    assert!(tags.len() > 0);
    let tags = get_tags_from_paths(paths);
    assert!(tags.len() > 0);
}
