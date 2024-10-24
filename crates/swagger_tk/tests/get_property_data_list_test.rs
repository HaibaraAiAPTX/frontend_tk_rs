use swagger_tk::gen::{get_correct_property_data_list_from_responses, get_property_data_from_request_body, get_property_data_list_from_parameters};
use utils::{get_open_api_object, get_path_item};

mod utils;

#[test]
fn get_property_data_list_from_parameters_test() {
    let open_api_object = get_open_api_object();
    let path_item = get_path_item(&open_api_object, "/pet/findByTags").unwrap();
    let list = get_property_data_list_from_parameters(&path_item.get.as_ref().unwrap().parameters.as_ref().unwrap());
    assert!(!list.is_empty(), "寻找参数失败");
    let path_item = get_path_item(&open_api_object, "/pet/{petId}").unwrap();
    let list = get_property_data_list_from_parameters(&path_item.post.as_ref().unwrap().parameters.as_ref().unwrap());
    assert!(!list.is_empty(), "寻找参数失败");
    let path_item = get_path_item(&open_api_object, "/pet/{petId}/uploadImage").unwrap();
    let list = get_property_data_from_request_body(&path_item.post.as_ref().unwrap().request_body.as_ref().unwrap());
    assert!(list.is_some(), "寻找请求体失败");
    let list = get_correct_property_data_list_from_responses(path_item.post.as_ref().unwrap().responses.as_ref());
    assert!(list.is_some(), "寻找返回模型失败");
    let list = get_property_data_list_from_parameters(&path_item.post.as_ref().unwrap().parameters.as_ref().unwrap());
    assert!(!list.is_empty(), "寻找参数失败");
    let path_item = get_path_item(&open_api_object, "/pet").unwrap();
    let list = get_property_data_from_request_body(&path_item.put.as_ref().unwrap().request_body.as_ref().unwrap());
    assert!(list.is_some(), "寻找请求体失败");
    let list = get_correct_property_data_list_from_responses(path_item.put.as_ref().unwrap().responses.as_ref());
    assert!(list.is_some(), "寻找返回模型失败");
    let path_item = get_path_item(&open_api_object, "/store/inventory").unwrap();
    let list = get_correct_property_data_list_from_responses(path_item.get.as_ref().unwrap().responses.as_ref());
    assert!(list.is_some(), "寻找返回模型失败");
}
