use std::path::PathBuf;

/// get swagger-tk-rs data dir
pub fn get_application_data_dir() -> Result<PathBuf, String> {
    dirs_next::data_dir()
        .map(|cache| cache.join("swagger-tk-rs"))
        .ok_or_else(|| "get swagger-tk-rs data dir failed".to_string())
}
