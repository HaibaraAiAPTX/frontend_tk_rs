use std::path::PathBuf;

pub fn get_application_data_dir() -> Result<PathBuf, String> {
    dirs_next::data_dir()
        .map(|cache| cache.join("frontend-tk-rs"))
        .ok_or_else(|| "get application data dir failed".to_string())
}
