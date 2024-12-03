use std::{env::current_dir, path::Path};

use swagger_tk::{gen::{AxiosTsGen, GenApi, TypescriptDeclarationGen}, model::OpenAPIObject};

fn main() {
    let open_api = get_open_api_object("打卡.json");
    // gen_api(&open_api);
    gen_model(&open_api);
}

fn get_open_api_object(filename: &str) -> OpenAPIObject {
    let file_path = current_dir().unwrap().join(filename);
    let text = std::fs::read_to_string(file_path).unwrap();
    OpenAPIObject::from_str(&text).unwrap()
}

#[allow(dead_code)]
fn gen_api(open_api: &OpenAPIObject) {
    let output = Path::new("D:\\Project\\TrainningAssistant\\Client\\apps\\backend\\src\\services\\TrainningAssistantMain");
    let mut axios_gen = AxiosTsGen::default();
    let apis = axios_gen.gen_apis(&open_api);
    if let Ok(apis) = apis {
        for (name, content) in apis {
            let file_path = output.join(format!("{}Service.ts", name));
            std::fs::write(file_path, content).unwrap();
        }
    }
}

#[allow(dead_code)]
fn gen_model(open_api: &OpenAPIObject) {
    let output = Path::new("D:\\Project\\TrainningAssistant\\Client\\typings");
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
