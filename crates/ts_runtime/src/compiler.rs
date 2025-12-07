use simple_bundler::SimpleBundler;
use std::path::PathBuf;

use crate::get_js_output_cache_dir;

pub fn compiler(entry_path: &PathBuf) -> Result<PathBuf, String> {
    let output_path = ensure_ts_runtime_cache_dir()?;
    let bundler = SimpleBundler::default();
    let entry_file_name = bundler.bundle(entry_path);
    bundler.write(&output_path);
    Ok(output_path.join(entry_file_name))
}

pub fn ensure_ts_runtime_cache_dir() -> Result<PathBuf, String> {
    let cache_dir = get_js_output_cache_dir()?;
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)
            .map_err(|err| format!("create ts runtime cache dir failed: {}", err))?;
    }
    Ok(cache_dir)
}
