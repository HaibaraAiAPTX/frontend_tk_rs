use swagger_tk::gen::get_property_data_list_from_parameters;
use utils::get_open_api_object;

mod utils;

#[test]
fn get_property_data_list_from_parameters_test() {
    let open_api_object = get_open_api_object();
    let paths = open_api_object.paths.unwrap();
    let (_path, path_item) = paths
        .iter()
        .find(|&(path, _path_item)| path == "/pet/findByTags")
        .unwrap();

    let list = get_property_data_list_from_parameters(&path_item.get.as_ref().unwrap().parameters);
    assert!(list.is_some(), "寻找参数失败");
    println!("{:#?}", list);
}
