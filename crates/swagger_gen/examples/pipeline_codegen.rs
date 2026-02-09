use std::{fs, path::PathBuf, str::FromStr};

use swagger_gen::pipeline::{
    generate_functions_contract_v1, generate_react_query_contract_v1,
    generate_vue_query_contract_v1,
};
use swagger_tk::model::OpenAPIObject;

fn main() {
    let mut args = std::env::args().skip(1);
    let input = args.next().unwrap_or_else(|| "3.1.0.json".to_string());
    let output_root = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("swagger-gen-pipeline-example"));

    let text = fs::read_to_string(&input).expect("read input openapi file fail");
    let open_api = OpenAPIObject::from_str(&text).expect("parse openapi fail");

    generate_functions_contract_v1(&open_api, &output_root).expect("generate functions fail");
    generate_react_query_contract_v1(&open_api, &output_root).expect("generate react-query fail");
    generate_vue_query_contract_v1(&open_api, &output_root).expect("generate vue-query fail");

    println!("generated to {}", output_root.display());
}
