use swagger_tk::getter::get_schema_name_list;
use utils::get_open_api_object;

mod utils;

#[test]
fn get_schema_name_list_test() {
    let open_api_object = get_open_api_object();
    let schema_list = get_schema_name_list(&open_api_object);
    assert!(schema_list.is_some());
    assert!(schema_list.unwrap().len() > 0);
}
