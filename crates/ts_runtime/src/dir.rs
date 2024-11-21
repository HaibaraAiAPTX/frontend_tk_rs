use common::get_application_data_dir;
use std::path::PathBuf;

pub fn get_js_output_cache_dir() -> Result<PathBuf, String> {
    get_application_data_dir()
        .map(|dir| dir.join("js-cache"))
        .map_err(|err| format!("get js compiler cache dir failed: {}", err))
}
