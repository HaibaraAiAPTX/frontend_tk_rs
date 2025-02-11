use swagger_tk::getter::get_schema_by_name;
use utils::get_open_api_object;

mod utils;

#[test]
fn get_schema_test() {
    let open_api_object = get_open_api_object(None);
    let schema = get_schema_by_name(&open_api_object, "Order");
    assert!(schema.is_some());
}
