use swagger_tk::gen::get_property_data_list_from_parameters;
use utils::{get_open_api_object, get_path_item};

mod utils;

#[test]
fn get_property_data_list_from_parameters_test() {
    let open_api_object = get_open_api_object();
    let path_item = get_path_item(&open_api_object, "/pet/findByTags").unwrap();
    let list = get_property_data_list_from_parameters(&path_item.get.as_ref().unwrap().parameters);
    assert!(list.is_some(), "寻找参数失败");
    let path_item = get_path_item(&open_api_object, "/pet/{petId}").unwrap();
    let list = get_property_data_list_from_parameters(&path_item.post.as_ref().unwrap().parameters);
    assert!(list.is_some(), "寻找参数失败");
}
