use std::env::current_dir;

use swagger_gen::{built_in_api_trait::GenApi, built_in_api::{AxiosTsGen, UniAppGen}};
use utils::get_open_api_object;

mod utils;

#[test]
fn gen_axios_apis_test() {
    println!("{:?}", current_dir());
    let open_api_object = get_open_api_object(None);
    let mut axios_gen = AxiosTsGen::default();
    let is_success = axios_gen.gen_apis(&open_api_object);
    assert!(is_success.is_ok());
    // for item in axios_gen.get_outputs().iter() {
    //     println!("{}：\n{}", item.0, item.1);
    // }
}

#[test]
fn gen_uni_apis_test() {
    let open_api_object = get_open_api_object(None);
    let mut uni_gen = UniAppGen::default();
    let is_success = uni_gen.gen_apis(&open_api_object);
    assert!(is_success.is_ok());
    for item in uni_gen.get_outputs().iter() {
        println!("{}：\n{}", item.0, item.1);
    }
}
