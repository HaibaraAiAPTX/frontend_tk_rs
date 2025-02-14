use std::{fs, path::Path};

pub fn ensure_path(p: &Path) {
  if !p.exists() {
    fs::create_dir_all(p).unwrap()
  }
}
