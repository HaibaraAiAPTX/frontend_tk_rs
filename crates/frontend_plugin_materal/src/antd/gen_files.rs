use std::{
    collections::HashMap,
    fs, iter,
    path::{Path, PathBuf},
};

use clap::{Error, Parser};
use path_clean::PathClean;
use swagger_gen::{core::ApiContext, utils::format_ts_code};
use swagger_tk::{
    getter::{get_paths_from_tag, get_schema_by_name},
    model::{OpenAPIObject, OperationObject, SchemaEnum},
};

#[derive(Parser, Debug)]
struct GenFileOpts {
    #[arg(long, default_value_t = String::from("src/services"))]
    service_dir: String,

    #[arg(long, default_value_t = String::from("src/hooks"))]
    hook_dir: String,

    #[arg(long, default_value_t = String::from("src/stores"))]
    store_dir: String,

    #[arg(long, default_value_t = String::from("src/utils"))]
    util_dir: String,

    #[arg(long, default_value_t = String::from("src"))]
    src_dir: String,

    #[arg(long, short)]
    all: Option<bool>,

    #[arg(long)]
    service: Option<bool>,

    #[arg(long, short)]
    hook: Option<bool>,

    #[arg(long)]
    store: Option<bool>,

    #[arg(long, short)]
    util: Option<bool>,

    #[arg(long)]
    app: Option<bool>,

    #[arg(long, short)]
    dependencies: Option<bool>,
}

pub fn gen_files(args: &[String], open_api: &OpenAPIObject) -> Result<(), Error> {
    let args = iter::once("--".to_string())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>();
    let opts = GenFileOpts::try_parse_from(args)?;

    let service_dir = Path::new(&opts.service_dir);
    let hook_dir = Path::new(&opts.hook_dir);
    let util_dir = Path::new(&opts.util_dir);
    let store_dir = Path::new(&opts.store_dir);
    let src_dir = Path::new(&opts.src_dir);

    let service = opts.all.or(opts.service).unwrap_or_default();
    let hook = opts.all.or(opts.hook).unwrap_or_default();
    let store = opts.all.or(opts.store).unwrap_or_default();
    let util = opts.all.or(opts.util).unwrap_or_default();
    let app = opts.all.or(opts.app).unwrap_or_default();
    let dependencies = opts.all.or(opts.dependencies).unwrap_or_default();

    let mut files = HashMap::<PathBuf, String>::new();

    if service {
        files.insert(
            service_dir.join("BaseService.ts").clean(),
            include_str!("base_api/BaseService.txt").to_string(),
        );
        files.insert(
            service_dir.join("ErrorHandler.ts").clean(),
            include_str!("base_api/ErrorHandler.txt").to_string(),
        );
        files.insert(
            service_dir.join("ErrorHandlerImp.ts").clean(),
            include_str!("base_api/ErrorHandlerImp.txt").to_string(),
        );
    }

    if hook {
        files.insert(
            hook_dir.join("layout.ts").clean(),
            include_str!("hooks/layout.txt").to_string(),
        );
        files.insert(
            hook_dir.join("modal-form.ts").clean(),
            include_str!("hooks/modal-form.txt").to_string(),
        );
    }

    if util {
        files.insert(
            util_dir.join("loop.ts").clean(),
            include_str!("utils/loop.txt").to_string(),
        );
        files.insert(
            util_dir.join("request.ts").clean(),
            include_str!("utils/request.txt").to_string(),
        );
        files.insert(
            util_dir.join("table.ts").clean(),
            include_str!("utils/table.txt").to_string(),
        );
        files.insert(
            util_dir.join("tree-map.ts").clean(),
            include_str!("utils/tree-map.txt").to_string(),
        );
        files.insert(
            util_dir.join("tree-utils.ts").clean(),
            include_str!("utils/tree-utils.txt").to_string(),
        );
    }

    if store {
        insert_store_dictionary_file(store_dir, &mut files, open_api);
    }

    if dependencies {
        files.insert(
            src_dir.join("dependencies.ts"),
            include_str!("dependencies.txt").to_string(),
        );
    }

    if app {
        files.insert(src_dir.join("app.tsx"), include_str!("app.txt").to_string());
    }

    // 写入文件
    files.iter().for_each(|(file_path, content)| {
        let dir = file_path.parent();
        if let Some(dir) = dir {
            if !dir.exists() {
                fs::create_dir_all(dir).unwrap();
            }
        }
        fs::write(file_path, content).unwrap();
    });

    Ok(())
}

/// 生成字典缓存
fn insert_store_dictionary_file(
    store_dir: &Path,
    files: &mut HashMap<PathBuf, String>,
    open_api: &OpenAPIObject,
) {
    let mut enum_list = Vec::<String>::new();
    let mut enum_name_map = HashMap::<String, &String>::new();
    let mut action_code = Vec::<String>::new();

    let apis = get_paths_from_tag(open_api, "Enums");
    for (&url, &path_item) in apis.iter() {
        if let Some(api_name) = url.split("GetAll").last() {
            let enum_name = {
                let mut chars = api_name.chars();
                let f = chars.next().unwrap().to_lowercase();
                let chars = chars.collect::<String>();
                format!("{f}{chars}")
            };
            enum_list.push(enum_name.clone());

            if let Some(operation) = &path_item.get {
                let api_context = ApiContext::new(url, "get", path_item, operation);
                let fn_name = &api_context.func_name;
                let description = &api_context
                    .description
                    .map(|s| format!("/** {s} */\n"))
                    .unwrap_or_default();

                if let Some(name) = get_enum_name(operation, open_api) {
                    enum_name_map.insert(enum_name.clone(), name);
                }

                action_code.push(format!(
                    r#"{description}async get{api_name}Options() {{
    return await initList(enumsService.{fn_name}.bind(enumsService), "{enum_name}")
}}"#
                ));
            }
        }
    }

    let store_type = enum_list
        .iter()
        .map(|name| {
            let description = enum_name_map
                .get(name)
                .map(|&v| format!("/** {v} */\n"))
                .unwrap_or_default();
            format!("{description}{name}: RequestOptionsType[]")
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let store_state = enum_list
        .iter()
        .map(|name| format!("{name}: []"))
        .collect::<Vec<_>>()
        .join(",\n");

    let store_action = action_code.join(",\n\n");

    let mut content = include_str!("store/dictionary.txt").to_string();
    content = content.replace("{{STORE_TYPE}}", store_type.as_str());
    content = content.replace("{{STORE_STATE}}", store_state.as_str());
    content = content.replace("{{STORE_ACTION}}", store_action.as_str());
    content = format_ts_code(&content).unwrap();

    files.insert(store_dir.join("dictionary.ts").clean(), content);
}

fn get_enum_name<'a>(
    operation: &'a OperationObject,
    open_api: &'a OpenAPIObject,
) -> Option<&'a String> {
    operation
        .responses
        .as_ref()
        .and_then(|v| v.get("200"))
        .and_then(|v| v.as_response())
        .and_then(|v| v.content.as_ref())
        .and_then(|v| v.get("application/json"))
        .and_then(|v| v.get_ref_schema_name())
        .and_then(|v| get_schema_by_name(open_api, v))
        .and_then(|v| v.get_object_property("Data"))
        .and_then(|v| v.get_array_item_ref_schema_name())
        .and_then(|v| get_schema_by_name(open_api, v))
        .and_then(|v| v.get_object_property("Key"))
        .and_then(|v| v.get_ref_schema_name())
        .and_then(|v| get_schema_by_name(open_api, v))
        .and_then(|v| match v {
            SchemaEnum::String(schema_string) => schema_string.description.as_ref(),
            SchemaEnum::Integer(schema_integer) => schema_integer.description.as_ref(),
            SchemaEnum::Number(schema_number) => schema_number.description.as_ref(),
            _ => None,
        })
}
