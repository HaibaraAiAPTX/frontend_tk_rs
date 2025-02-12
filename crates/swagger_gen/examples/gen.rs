use std::fs;
use std::{env::current_dir, vec};

use path_clean::PathClean;
use swagger_gen::build_in_api_trait::GenApi;
use swagger_gen::built_in_api::AxiosTsGen;
use swagger_gen::built_in_declaration::TypescriptDeclarationGen;
use swagger_tk::model::OpenAPIObject;

fn main() {
    let open_api = get_open_api_object("打卡.json");
    gen_api(&open_api);
    gen_model(&open_api);
}

fn get_open_api_object(filename: &str) -> OpenAPIObject {
    let file_path = current_dir().unwrap().join(filename);
    let text = std::fs::read_to_string(file_path).unwrap();
    OpenAPIObject::from_str(&text).unwrap()
}

#[allow(dead_code)]
fn gen_api(open_api: &OpenAPIObject) {
    let outputs = vec![
        current_dir().unwrap().join("./crates/swagger_gen/examples/services").clean()
    ];
    let mut axios_gen = AxiosTsGen::default();
    axios_gen.set_open_api(&open_api);
    axios_gen.gen_apis(&open_api).unwrap();
    for (name, content) in axios_gen.get_outputs() {
        for output in &outputs {
            if !output.exists() {
                fs::create_dir_all(output).unwrap()
            }
            let file_path = output.join(format!("{}Service.ts", name));
            std::fs::write(file_path, content.clone()).unwrap();
        }
    }
}

#[allow(dead_code)]
fn gen_model(open_api: &OpenAPIObject) {
    let output = current_dir().unwrap().join("./crates/swagger_gen/examples/typings").clean();
    if !output.exists() {
        fs::create_dir_all(&output).unwrap()
    }
    let model_gen = TypescriptDeclarationGen {
        open_api,
    };

    let models = model_gen.gen_declarations();
    if let Ok(models) = models {
        for (name, content) in models {
            let file_path = output.join(name);
            std::fs::write(file_path, content).unwrap();
        }
    }
}
