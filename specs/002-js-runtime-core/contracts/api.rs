// Rust API Contract for ts_runtime

use std::path::PathBuf;
use quickjs_runtime::values::JsValueFacade;
use quickjs_runtime::quickjsvalueadapter::QuickJsValueAdapter;

/// Executes a script file directly.
/// 
/// # Arguments
/// * `main_file_path` - Path to the entry point script (TS or JS).
/// 
/// # Returns
/// * `Result<JsValueFacade, String>` - The result of the execution. 
///   If the script returns a Promise, it is awaited and the resolved value is returned.
pub async fn run_main_file(main_file_path: &PathBuf) -> Result<JsValueFacade, String>;

/// Executes a specific function exported by a script.
/// 
/// # Arguments
/// * `main_file_path` - Path to the entry point script.
/// * `func_name` - Name of the exported function to call.
/// * `get_args` - Closure that provides arguments for the function.
/// 
/// # Returns
/// * `Result<JsValueFacade, String>` - The return value of the function.
///   If the function returns a Promise, it is awaited and the resolved value is returned.
pub async fn run_func(
    main_file_path: &PathBuf,
    func_name: String,
    get_args: impl Fn() -> Vec<QuickJsValueAdapter> + Send + 'static,
) -> Result<JsValueFacade, String>;
