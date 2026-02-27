use std::{cell::RefCell, collections::HashMap, path::Path};

use clap::Parser;
use swagger_gen::pipeline::{CodegenPipeline, FileSystemWriter};
use swagger_gen_aptx::{
  AptxFunctionsRenderer, AptxQueryMutationPass, AptxReactQueryRenderer, AptxVueQueryRenderer,
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

  #[arg(long)]
  model_mode: Option<String>,

  #[arg(long)]
  model_path: Option<String>,
}

pub type TerminalGenerator = Box<dyn for<'a> Fn(&'a OpenAPIObject, &'a Path) -> Result<(), String>>;

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
    self
      .generators
      .borrow_mut()
      .insert(id.to_string(), generator);
  }

  pub fn generate(&self, id: &str, open_api: &OpenAPIObject, output: &Path) -> Result<(), String> {
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

/// Build client import configuration from command-line options
fn build_client_import_config(
  client_mode: Option<&str>,
  client_path: Option<&str>,
  client_package: Option<&str>,
  client_import_name: Option<&str>,
) -> Option<swagger_gen::pipeline::ClientImportConfig> {
  match client_mode {
    None => None,
    Some(mode) => Some(swagger_gen::pipeline::ClientImportConfig {
      mode: mode.to_string(),
      client_path: client_path.map(|s| s.to_string()),
      client_package: client_package.map(|s| s.to_string()),
      import_name: client_import_name.map(|s| s.to_string()),
    }),
  }
}

/// Build model import configuration from command-line options
fn build_model_import_config(
  model_mode: Option<&str>,
  model_path: Option<&str>,
) -> Option<swagger_gen::pipeline::ModelImportConfig> {
  match model_mode {
    None => None,
    Some("package") => Some(swagger_gen::pipeline::ModelImportConfig {
      import_type: "package".to_string(),
      package_path: Some(model_path.unwrap_or("@my-org/models").to_string()),
      relative_path: None,
      original_path: model_path.map(|s| s.to_string()),
    }),
    Some("relative") => Some(swagger_gen::pipeline::ModelImportConfig {
      import_type: "relative".to_string(),
      package_path: None,
      relative_path: None,
      original_path: model_path.map(|s| s.to_string()),
    }),
    _ => None,
  }
}

fn create_builtin_registry_with_options(
  client_mode: Option<String>,
  client_path: Option<String>,
  client_package: Option<String>,
  client_import_name: Option<String>,
  model_mode: Option<String>,
  model_path: Option<String>,
) -> TerminalRegistry {
  let registry = TerminalRegistry::new();

  // Clone the strings for use in closures
  let client_mode_clone = client_mode.clone();
  let client_path_clone = client_path.clone();
  let client_package_clone = client_package.clone();
  let client_import_name_clone = client_import_name.clone();
  let model_mode_clone = model_mode.clone();
  let model_path_clone = model_path.clone();

  registry.register(
    "functions",
    Box::new(move |open_api, output| {
      let client_import = build_client_import_config(
        client_mode_clone.as_deref(),
        client_path_clone.as_deref(),
        client_package_clone.as_deref(),
        client_import_name_clone.as_deref(),
      );
      let model_import = build_model_import_config(
        model_mode_clone.as_deref(),
        model_path_clone.as_deref(),
      );

      let pipeline = CodegenPipeline::default()
        .with_transform(Box::new(AptxQueryMutationPass))
        .with_client_import(client_import)
        .with_model_import(model_import)
        .with_renderer(Box::new(AptxFunctionsRenderer))
        .with_writer(Box::new(FileSystemWriter::new(output)));

      pipeline.plan(open_api).map(|_| ())
    }),
  );

  // Clone again for the next registry
  let client_mode_clone = client_mode.clone();
  let client_path_clone = client_path.clone();
  let client_package_clone = client_package.clone();
  let client_import_name_clone = client_import_name.clone();
  let model_mode_clone = model_mode.clone();
  let model_path_clone = model_path.clone();

  registry.register(
    "react-query",
    Box::new(move |open_api, output| {
      let client_import = build_client_import_config(
        client_mode_clone.as_deref(),
        client_path_clone.as_deref(),
        client_package_clone.as_deref(),
        client_import_name_clone.as_deref(),
      );
      let model_import = build_model_import_config(
        model_mode_clone.as_deref(),
        model_path_clone.as_deref(),
      );

      let pipeline = CodegenPipeline::default()
        .with_transform(Box::new(AptxQueryMutationPass))
        .with_client_import(client_import)
        .with_model_import(model_import)
        .with_renderer(Box::new(AptxReactQueryRenderer))
        .with_writer(Box::new(FileSystemWriter::new(output)));

      pipeline.plan(open_api).map(|_| ())
    }),
  );

  // Clone again for the next registry
  let client_mode_clone = client_mode;
  let client_path_clone = client_path;
  let client_package_clone = client_package;
  let client_import_name_clone = client_import_name;
  let model_mode_clone = model_mode;
  let model_path_clone = model_path;

  registry.register(
    "vue-query",
    Box::new(move |open_api, output| {
      let client_import = build_client_import_config(
        client_mode_clone.as_deref(),
        client_path_clone.as_deref(),
        client_package_clone.as_deref(),
        client_import_name_clone.as_deref(),
      );
      let model_import = build_model_import_config(
        model_mode_clone.as_deref(),
        model_path_clone.as_deref(),
      );

      let pipeline = CodegenPipeline::default()
        .with_transform(Box::new(AptxQueryMutationPass))
        .with_client_import(client_import)
        .with_model_import(model_import)
        .with_renderer(Box::new(AptxVueQueryRenderer))
        .with_writer(Box::new(FileSystemWriter::new(output)));

      pipeline.plan(open_api).map(|_| ())
    }),
  );

  registry
}

thread_local! {
  static BUILTIN_REGISTRY: TerminalRegistry = create_builtin_registry_with_options(None, None, None, None, None, None);
}

pub fn run_terminal_codegen(args: &[String], open_api: &OpenAPIObject) {
  let result = (|| -> Result<(), String> {
    let args: Vec<String> = std::iter::once("--".to_string())
      .chain(args.iter().cloned())
      .collect();
    let options =
      TerminalCodegenOps::try_parse_from(args).map_err(|e| format!("Invalid arguments: {e}"))?;
    let output = Path::new(&options.output);

    // Create registry with client import options if provided
    let registry = create_builtin_registry_with_options(
      options.client_mode,
      options.client_path,
      options.client_package,
      options.client_import_name,
      options.model_mode,
      options.model_path,
    );

    registry.generate(&options.terminal, open_api, output)
  })();

  if let Err(e) = result {
    panic!("terminal:codegen failed: {e}");
  }
}
