use std::{
  cell::RefCell,
  collections::HashMap,
  path::Path,
};

use clap::Parser;
use swagger_gen::pipeline::{
  generate_axios_js_v1, generate_axios_ts_v1, generate_functions_contract_v1,
  generate_functions_contract_v1_with_client, generate_react_query_contract_v1,
  generate_react_query_contract_v1_with_client, generate_uniapp_v1, generate_vue_query_contract_v1,
  generate_vue_query_contract_v1_with_client,
};
use swagger_tk::model::OpenAPIObject;

#[derive(Debug, Clone, Parser)]
pub struct TerminalCodegenOps {
  #[arg(long)]
  terminal: String,

  #[arg(long)]
  output: String,

  #[arg(long)]
  client_mode: Option<String>,

  #[arg(long)]
  client_path: Option<String>,

  #[arg(long)]
  client_package: Option<String>,

  #[arg(long)]
  client_import_name: Option<String>,
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

fn create_builtin_registry_with_options(
  client_mode: Option<String>,
  client_path: Option<String>,
  client_package: Option<String>,
  client_import_name: Option<String>,
) -> TerminalRegistry {
  let registry = TerminalRegistry::new();

  // Clone the strings for use in closures
  let client_mode_clone = client_mode.clone();
  let client_path_clone = client_path.clone();
  let client_package_clone = client_package.clone();
  let client_import_name_clone = client_import_name.clone();

  // Check if any client config is provided
  let has_client_config = client_mode.is_some() || client_path.is_some() || client_package.is_some() || client_import_name.is_some();

  registry.register("functions", Box::new(move |open_api, output| {
    if has_client_config {
      generate_functions_contract_v1_with_client(
        open_api,
        output,
        client_mode_clone.as_deref(),
        client_path_clone.as_deref(),
        client_package_clone.as_deref(),
        client_import_name_clone.as_deref(),
      ).map(|_| ())
    } else {
      generate_functions_contract_v1(open_api, output).map(|_| ())
    }
  }));

  // Clone again for the next registry
  let client_mode_clone = client_mode.clone();
  let client_path_clone = client_path.clone();
  let client_package_clone = client_package.clone();
  let client_import_name_clone = client_import_name.clone();

  registry.register("react-query", Box::new(move |open_api, output| {
    if has_client_config {
      generate_react_query_contract_v1_with_client(
        open_api,
        output,
        client_mode_clone.as_deref(),
        client_path_clone.as_deref(),
        client_package_clone.as_deref(),
        client_import_name_clone.as_deref(),
      ).map(|_| ())
    } else {
      generate_react_query_contract_v1(open_api, output).map(|_| ())
    }
  }));

  // Clone again for the next registry
  let client_mode_clone = client_mode;
  let client_path_clone = client_path;
  let client_package_clone = client_package;
  let client_import_name_clone = client_import_name;

  registry.register("vue-query", Box::new(move |open_api, output| {
    if has_client_config {
      generate_vue_query_contract_v1_with_client(
        open_api,
        output,
        client_mode_clone.as_deref(),
        client_path_clone.as_deref(),
        client_package_clone.as_deref(),
        client_import_name_clone.as_deref(),
      ).map(|_| ())
    } else {
      generate_vue_query_contract_v1(open_api, output).map(|_| ())
    }
  }));

  // Other terminals (axios-ts, axios-js, uniapp) don't use client imports
  registry.register("axios-ts", Box::new(|open_api, output| {
    generate_axios_ts_v1(open_api, output).map(|_| ())
  }));
  registry.register("axios-js", Box::new(|open_api, output| {
    generate_axios_js_v1(open_api, output).map(|_| ())
  }));
  registry.register("uniapp", Box::new(|open_api, output| {
    generate_uniapp_v1(open_api, output).map(|_| ())
  }));

  registry
}

thread_local! {
  static BUILTIN_REGISTRY: TerminalRegistry = create_builtin_registry_with_options(None, None, None, None);
}

pub fn run_terminal_codegen(args: &[String], open_api: &OpenAPIObject) {
  let result = (|| -> Result<(), String> {
    let args: Vec<String> = std::iter::once("--".to_string())
      .chain(args.iter().cloned())
      .collect();
    let options = TerminalCodegenOps::try_parse_from(args)
      .map_err(|e| format!("Invalid arguments: {e}"))?;
    let output = Path::new(&options.output);

    // Create registry with client import options if provided
    let registry = create_builtin_registry_with_options(
      options.client_mode,
      options.client_path,
      options.client_package,
      options.client_import_name,
    );

    registry.generate(&options.terminal, open_api, output)
  })();

  if let Err(e) = result {
    panic!("terminal:codegen failed: {e}");
  }
}
