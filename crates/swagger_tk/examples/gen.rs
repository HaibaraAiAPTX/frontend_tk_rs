use std::{env::current_dir, path::Path, vec};

use swagger_tk::{gen::{AxiosTsGen, GenApi, TypescriptDeclarationGen}, model::OpenAPIObject};

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
        Path::new("F:\\工作室\\TrainningAssistant\\Client\\apps\\backend\\src\\services\\TrainningAssistantMain"),
        Path::new("F:\\工作室\\TrainningAssistant\\Client\\apps\\wechat-account-offical\\src\\services")
    ];
    let mut axios_gen = AxiosTsGen::default();
    axios_gen.set_open_api(&open_api);
    let apis = axios_gen.gen_apis(&open_api);
    if let Ok(apis) = apis {
        for (name, content) in apis {
            for output in &outputs {
                let file_path = output.join(format!("{}Service.ts", name));
                std::fs::write(file_path, content.clone()).unwrap();
            }
        }
    }
}

#[allow(dead_code)]
fn gen_model(open_api: &OpenAPIObject) {
    let output = Path::new("F:\\工作室\\TrainningAssistant\\Client\\typings");
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
