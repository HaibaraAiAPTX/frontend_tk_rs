use swagger_tk::{model::OpenAPIObject, utils::{get_tags_from_open_api, get_tags_from_path_item}};

mod utils;

fn main() {
    let swagger_text = utils::get_json_file("3.1.0.json").expect("获取文件失败");
    let open_api_object = OpenAPIObject::from_str(&swagger_text).expect("解析文件失败");

    open_api_object.paths.iter().for_each(|item| {
        item.iter().for_each(|(path, item)| {
            let tags = get_tags_from_path_item(&item);
            println!("{} {:?}", path, tags);
        });
    });

    let tags = get_tags_from_open_api(&open_api_object);
    if let Some(tags) = tags {
        println!("{:#?}", tags)
    }
}
