use std::path::PathBuf;

use path_clean::PathClean;
use quickjs_runtime::jsutils::modules::ScriptModuleLoader;

pub struct CustomModuleLoader;

impl ScriptModuleLoader for CustomModuleLoader {
    fn normalize_path(
        &self,
        _realm: &quickjs_runtime::quickjsrealmadapter::QuickJsRealmAdapter,
        ref_path: &str,
        path: &str,
    ) -> Option<String> {
        let path_buf = PathBuf::from(ref_path);
        let dir = path_buf.parent()?;
        Some(dir.join(path).clean().to_string_lossy().to_string())
    }

    fn load_module(
        &self,
        _realm: &quickjs_runtime::quickjsrealmadapter::QuickJsRealmAdapter,
        absolute_path: &str,
    ) -> String {
        println!("load module: {}", absolute_path);
        std::fs::read_to_string(absolute_path)
            .map_err(|err| format!("read module file failed: {}, path: {}", err, absolute_path))
            .unwrap_or_default()
    }
}
