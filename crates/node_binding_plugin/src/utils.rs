use std::{collections::HashMap, fs, path::Path};

pub fn ensure_path(p: &Path) {
    if !p.exists() {
        fs::create_dir_all(p).unwrap()
    }
}

pub fn create_all_file(dir: &Path, files: &HashMap<String, String>) {
    files
        .iter()
        .for_each(|(k, v)| fs::write(dir.join(k), v).unwrap());
}
