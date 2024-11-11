use crate::compile;
use deno_core::{
    anyhow::Result,
    serde_json, serde_v8,
    v8::{self, Global},
    FsModuleLoader, ModuleSpecifier,
};
use deno_runtime::{
    deno_fs::RealFs,
    deno_permissions::PermissionsContainer,
    permissions::RuntimePermissionDescriptorParser,
    worker::{MainWorker, WorkerOptions, WorkerServiceOptions},
};
use std::{path::PathBuf, rc::Rc, sync::Arc};

pub enum ResultType {
    Value(Global<v8::Value>),
    Promise(Global<v8::Promise>),
}

pub async fn run_main_file(main_file_path: &PathBuf) -> Result<serde_json::Value, String> {
    let compiled_file_path = get_compiled_file_path(main_file_path).unwrap();
    let main_module = ModuleSpecifier::from_file_path(&compiled_file_path).unwrap();
    let mut worker = create_worker(main_module);
    let module_specifier = ModuleSpecifier::from_file_path(compiled_file_path).unwrap();
    let module_id = worker
        .js_runtime
        .load_main_es_module(&module_specifier)
        .await
        .unwrap();
    let f = worker.js_runtime.mod_evaluate(module_id);
    f.await.unwrap();
    worker.run_event_loop(false).await.unwrap();
    let context = worker.js_runtime.main_context();
    let scope = &mut worker.js_runtime.handle_scope();
    let context_local = v8::Local::new(scope, context);
    let global = context_local.global(scope);
    let result_key = v8::String::new(scope, "result").unwrap();
    let result = global.get(scope, result_key.into()).unwrap();
    let deserialized_value = serde_v8::from_v8::<serde_json::Value>(scope, result.into())
        .map_err(|e| format!("failed to deserialize value: {}", e))?;
    Ok(deserialized_value)
}

pub async fn run_func(
    main_file_path: &PathBuf,
    func_name: &str,
) -> Result<serde_json::Value, String> {
    let compiled_file_path = get_compiled_file_path(main_file_path).unwrap();
    let specifier = ModuleSpecifier::from_file_path(&compiled_file_path).unwrap();
    let mut worker = create_worker(specifier.clone());

    let module_id = worker
        .js_runtime
        .load_main_es_module(&specifier)
        .await
        .unwrap();

    let f = worker.js_runtime.mod_evaluate(module_id);
    let module_namespace = worker.js_runtime.get_module_namespace(module_id).unwrap();
    f.await.unwrap();

    let promise = {
        let scope: &mut v8::HandleScope<'_> = &mut worker.js_runtime.handle_scope();
        let module_namespace = v8::Local::new(scope, &module_namespace);

        let key = v8::String::new(scope, func_name).unwrap();
        let value = module_namespace.get(scope, key.into()).unwrap();

        let js_function = v8::Local::<v8::Function>::try_from(value)
            .map_err(|_| format!("{} is not a function", func_name))?;

        let result = js_function
            .call(scope, module_namespace.into(), &[])
            .ok_or_else(|| "function call failed".to_string())?;
        if !result.is_promise() {
            return serde_v8::from_v8::<serde_json::Value>(scope, result)
                .map_err(|e| format!("Failed to deserialize value: {}", e));
        } else {
            let promise = v8::Local::<v8::Promise>::try_from(result).unwrap();
            v8::Global::new(scope, promise)
        }
    };

    worker.run_event_loop(Default::default()).await.unwrap();
    let scope: &mut v8::HandleScope<'_> = &mut worker.js_runtime.handle_scope();
    let promise = v8::Local::new(scope, &promise);
    match promise.state() {
        v8::PromiseState::Fulfilled => {
            let value = promise.result(scope);
            serde_v8::from_v8::<serde_json::Value>(scope, value)
                .map_err(|e| format!("Failed to deserialize value: {}", e))
        }
        v8::PromiseState::Rejected => {
            let error = promise.result(scope);
            let error_str = error.to_rust_string_lossy(scope);
            Err(error_str)
        }
        v8::PromiseState::Pending => Err("promise is still pending".to_string()),
    }
}

fn create_worker(main_module: ModuleSpecifier) -> MainWorker {
    let fs = Arc::new(RealFs);
    let permission_desc_parser = Arc::new(RuntimePermissionDescriptorParser::new(fs.clone()));
    MainWorker::bootstrap_from_options(
        main_module.clone(),
        WorkerServiceOptions {
            module_loader: Rc::new(FsModuleLoader),
            fs,
            permissions: PermissionsContainer::allow_all(permission_desc_parser),
            blob_store: Default::default(),
            broadcast_channel: Default::default(),
            feature_checker: Default::default(),
            node_services: Default::default(),
            npm_process_state_provider: Default::default(),
            root_cert_store_provider: Default::default(),
            shared_array_buffer_store: Default::default(),
            compiled_wasm_module_store: Default::default(),
            v8_code_cache: Default::default(),
        },
        WorkerOptions {
            ..Default::default()
        },
    )
}

fn get_compiled_file_path(main_file_path: &PathBuf) -> Result<PathBuf, String> {
    let compiled_file_path = compile(main_file_path).unwrap();
    if !compiled_file_path.exists() {
        return Err(format!(
            "compiled file not found: {}",
            compiled_file_path.display()
        ));
    }
    Ok(compiled_file_path)
}
