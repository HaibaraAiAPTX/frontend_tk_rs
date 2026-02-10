use std::path::Path;

use clap::Parser;
use swagger_gen::pipeline::{
  generate_axios_js_v1, generate_axios_ts_v1, generate_functions_contract_v1,
  generate_react_query_contract_v1, generate_uniapp_v1, generate_vue_query_contract_v1,
};
use swagger_tk::model::OpenAPIObject;

#[derive(Debug, Clone, Parser)]
pub struct TerminalCodegenOps {
  #[arg(long)]
  terminal: String,

  #[arg(long)]
  output: String,
}

pub fn run_terminal_codegen(args: &[String], open_api: &OpenAPIObject) {
  let args: Vec<String> = std::iter::once("--".to_string())
    .chain(args.iter().cloned())
    .collect();
  let options = TerminalCodegenOps::try_parse_from(args).unwrap();
  let output = Path::new(&options.output);

  match options.terminal.as_str() {
    "axios-ts" => {
      generate_axios_ts_v1(open_api, output).unwrap();
    }
    "axios-js" => {
      generate_axios_js_v1(open_api, output).unwrap();
    }
    "uniapp" => {
      generate_uniapp_v1(open_api, output).unwrap();
    }
    "functions" => {
      generate_functions_contract_v1(open_api, output).unwrap();
    }
    "react-query" => {
      generate_react_query_contract_v1(open_api, output).unwrap();
    }
    "vue-query" => {
      generate_vue_query_contract_v1(open_api, output).unwrap();
    }
    _ => panic!("unsupported terminal: {}", options.terminal),
  }
}
