use std::time::Instant;

use swagger_tk::{
    getter::{get_paths_from_tag, get_schema, get_schema_name_list, get_tags},
    model::OpenAPIObject,
};

mod utils;

fn main() {
    let time = Instant::now();
    let swagger_text = utils::get_json_file("打卡.json").expect("获取文件失败");
    let open_api_object = OpenAPIObject::from_str(&swagger_text).expect("解析文件失败");

    println!("请求 api 列表：");
    open_api_object
        .paths
        .iter()
        .flat_map(|item| item.iter())
        .for_each(|(path, _)| {
            println!("{}", path);
        });

    println!("请求 controller 列表：");
    get_tags(&open_api_object).iter().for_each(|&tag| {
        println!("{}", tag);
    });

    println!("请求 schema 列表：");
    get_schema_name_list(&open_api_object).map(|schemas| {
        schemas.iter().for_each(|schema| {
            println!("{}", schema);
        });
    });

    let list = get_paths_from_tag(&open_api_object, "User");
    println!("{:#?}", list.keys());

    println!("测试获取模型：");
    let schema = get_schema(&open_api_object, "QueryUserRequestModel");
    println!("{:#?}", schema);
    println!("用时：{} 毫秒", time.elapsed().as_millis());
}
