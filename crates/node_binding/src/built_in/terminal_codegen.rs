use std::{
  cell::RefCell,
  collections::HashMap,
  path::Path,
};

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

pub type TerminalGenerator =
  Box<dyn for<'a> Fn(&'a OpenAPIObject, &'a Path) -> Result<(), String>>;

pub struct TerminalRegistry {
  generators: RefCell<HashMap<String, TerminalGenerator>>,
}

impl TerminalRegistry {
  pub fn new() -> Self {
    Self {
      generators: RefCell::new(HashMap::new()),
    }
  }

  pub fn register(&self, id: &str, generator: TerminalGenerator) {
    self.generators.borrow_mut().insert(id.to_string(), generator);
  }

  pub fn generate(
    &self,
    id: &str,
    open_api: &OpenAPIObject,
    output: &Path,
  ) -> Result<(), String> {
    let generators = self.generators.borrow();
    let generator = generators
      .get(id)
      .ok_or(format!("unsupported terminal: {id}"))?;
    generator(open_api, output)
  }
}

impl Default for TerminalRegistry {
  fn default() -> Self {
    Self::new()
  }
}

fn create_builtin_registry() -> TerminalRegistry {
  let registry = TerminalRegistry::new();
  registry.register("axios-ts", Box::new(|open_api, output| {
    generate_axios_ts_v1(open_api, output).map(|_| ())
  }));
  registry.register("axios-js", Box::new(|open_api, output| {
    generate_axios_js_v1(open_api, output).map(|_| ())
  }));
  registry.register("uniapp", Box::new(|open_api, output| {
    generate_uniapp_v1(open_api, output).map(|_| ())
  }));
  registry.register("functions", Box::new(|open_api, output| {
    generate_functions_contract_v1(open_api, output).map(|_| ())
  }));
  registry.register("react-query", Box::new(|open_api, output| {
    generate_react_query_contract_v1(open_api, output).map(|_| ())
  }));
  registry.register("vue-query", Box::new(|open_api, output| {
    generate_vue_query_contract_v1(open_api, output).map(|_| ())
  }));
  registry
}

thread_local! {
  static BUILTIN_REGISTRY: TerminalRegistry = create_builtin_registry();
}

pub fn run_terminal_codegen(args: &[String], open_api: &OpenAPIObject) {
  let result = (|| -> Result<(), String> {
    let args: Vec<String> = std::iter::once("--".to_string())
      .chain(args.iter().cloned())
      .collect();
    let options = TerminalCodegenOps::try_parse_from(args)
      .map_err(|e| format!("Invalid arguments: {e}"))?;
    let output = Path::new(&options.output);

    BUILTIN_REGISTRY
      .with(|registry| registry.generate(&options.terminal, open_api, output))
  })();

  if let Err(e) = result {
    panic!("terminal:codegen failed: {e}");
  }
}
