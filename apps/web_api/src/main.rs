use swagger_tk::model::OpenAPIObject;

fn main() {
    let open_api = OpenAPIObject::from_str(r#"{
        "openapi": "3.0.0",
        "info": {
            "title": "测试",
            "version": "1.0.0"
        }
    }"#).unwrap();

    println!("{:#?}", open_api);
}
