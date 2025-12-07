use quickjs_runtime::{
    builder::QuickJsRuntimeBuilder, facades::QuickJsRuntimeFacade, jsutils::JsError,
    jsutils::Script, quickjsrealmadapter::QuickJsRealmAdapter,
    quickjsvalueadapter::QuickJsValueAdapter, values::JsValueFacade,
};

use crate::{CustomModuleLoader, compiler};
use std::{fs, path::PathBuf};

fn create_runtime() -> QuickJsRuntimeFacade {
    QuickJsRuntimeBuilder::new()
        .script_module_loader(CustomModuleLoader)
        .build()
}

/// Runs the main file of a TypeScript/JavaScript project.
///
/// This function bundles the project starting from `main_file_path`,
/// executes the bundle, and awaits any top-level promises.
pub async fn run_main_file(main_file_path: &PathBuf) -> Result<JsValueFacade, String> {
    let compiled_file_path = get_compiled_file_path(main_file_path)?;
    let compiled_file_str = compiled_file_path
        .to_str()
        .ok_or("invalid compiled file path")?;
    let file_content = fs::read_to_string(compiled_file_str)
        .map_err(|err| format!("read compiled file failed: {}", err))?;

    let rt = create_runtime();
    let res = rt
        .eval_module(None, Script::new(compiled_file_str, &file_content))
        .await
        .map_err(|err| format!("run main file failed: {}", err))?;
    if let JsValueFacade::JsPromise { cached_promise } = res {
        let res = cached_promise
            .get_promise_result()
            .await
            .map_err(|err| format!("promise then failed: {}", err))?
            .map_err(|_| "promise catch failed")?;
        return Ok(res);
    }
    Ok(res)
}

/// Invokes a specific exported function from a TypeScript/JavaScript module.
///
/// This function bundles the project, executes the module, and then invokes
/// the specified function `func_name` with arguments provided by `get_args`.
/// It handles async functions by awaiting the returned Promise.
pub async fn run_func(
    main_file_path: &PathBuf,
    func_name: String,
    get_args: impl Fn(&QuickJsRealmAdapter) -> Vec<QuickJsValueAdapter> + Send + 'static,
) -> Result<JsValueFacade, String> {
    let compiled_file_path = get_compiled_file_path(main_file_path)?;
    let compiled_file_str = compiled_file_path
        .to_str()
        .ok_or("invalid compiled file path")?;
    let file_content = fs::read_to_string(compiled_file_str)
        .map_err(|err| format!("read compiled file failed: {}", err))?;

    let rt = create_runtime();
    let res = rt
        .eval_module(None, Script::new(compiled_file_str, &file_content))
        .await
        .map_err(|err| format!("run module file failed: {}", err))?;

    if let JsValueFacade::JsPromise { cached_promise } = res {
        cached_promise
            .get_promise_result()
            .await
            .map_err(|err| format!("promise then failed: {}", err))?
            .map_err(|_| "promise catch failed")?;
    }

    // Run helper to get namespace
    let import_path = compiled_file_str.replace("\\", "/");
    let helper_script = format!(
        "import * as ns from '{}'; globalThis.__temp_ns = ns;",
        import_path
    );

    let res = rt
        .eval_module(None, Script::new("helper.js", &helper_script))
        .await
        .map_err(|err| format!("run helper script failed: {}", err))?;

    if let JsValueFacade::JsPromise { cached_promise } = res {
        cached_promise
            .get_promise_result()
            .await
            .map_err(|err| format!("helper promise then failed: {}", err))?
            .map_err(|_| "helper promise catch failed")?;
    }

    let res = rt
        .loop_realm(None, move |_, realm| {
            let args = get_args(realm);
            let args_refs: Vec<&QuickJsValueAdapter> = args.iter().collect();

            let global = realm.get_global()?;
            let module_ns_adapter =
                realm
                    .get_object_property(&global, "__temp_ns")
                    .map_err(|err| {
                        JsError::new(
                            "Error".to_string(),
                            format!("get __temp_ns failed: {}", err),
                            String::new(),
                        )
                    })?;

            let func = realm
                .get_object_property(&module_ns_adapter, &func_name)
                .map_err(|err| {
                    JsError::new(
                        "Error".to_string(),
                        format!("get function {} failed: {}", func_name, err),
                        String::new(),
                    )
                })?;

            let result_adapter = realm.invoke_function(None, &func, &args_refs)?;
            realm.to_js_value_facade(&result_adapter)
        })
        .await
        .map_err(|err| format!("invoke function failed: {}", err))?;

    if let JsValueFacade::JsPromise { cached_promise } = res {
        let res = cached_promise
            .get_promise_result()
            .await
            .map_err(|err| format!("promise then failed: {}", err))?
            .map_err(|_| "promise catch failed")?;
        return Ok(res);
    }
    Ok(res)
}

fn get_compiled_file_path(main_file_path: &PathBuf) -> Result<PathBuf, String> {
    let compiled_file_path = compiler(main_file_path).unwrap();
    if !compiled_file_path.exists() {
        return Err(format!(
            "compiled file not found: {}",
            compiled_file_path.display()
        ));
    }
    Ok(compiled_file_path)
}
