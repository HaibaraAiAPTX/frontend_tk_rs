use swagger_tk::gen::{GenApi, UniAppGen};
use utils::get_open_api_object;
mod utils;

#[test]
fn gen_apis_test() {
    let open_api_object = get_open_api_object(None);
    let mut uni_gen = UniAppGen::default();
    let apis = uni_gen.gen_apis(&open_api_object);
    assert!(apis.is_ok());
    for item in apis.unwrap().iter() {
        println!("{}：\n{}", item.0, item.1);
    }
}
