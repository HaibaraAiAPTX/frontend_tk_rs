use std::{
  fs::OpenOptions,
  path::{Path, PathBuf},
};

use aptx_frontend_tk_binding_plugin::utils::ensure_path;
use fs2::FileExt;

const OUTPUT_LOCK_FILE_NAME: &str = ".aptx-codegen.lock";

pub struct OutputLockGuard {
  file: std::fs::File,
  #[allow(dead_code)]
  path: PathBuf,
}

pub fn lock_output_root(output: &Path) -> Result<OutputLockGuard, String> {
  ensure_path(output);

  let path = output.join(OUTPUT_LOCK_FILE_NAME);
  let file = OpenOptions::new()
    .create(true)
    .read(true)
    .write(true)
    .open(&path)
    .map_err(|err| format!("Failed to open output lock file {path:?}: {err}"))?;

  file
    .lock_exclusive()
    .map_err(|err| format!("Failed to lock output directory {output:?}: {err}"))?;

  Ok(OutputLockGuard { file, path })
}

impl Drop for OutputLockGuard {
  fn drop(&mut self) {
    let _ = self.file.unlock();
  }
}

#[cfg(test)]
mod tests {
  use std::fs::OpenOptions;

  use fs2::FileExt;
  use tempfile::TempDir;

  use super::lock_output_root;

  #[test]
  fn lock_output_root_blocks_a_second_exclusive_lock() {
    let temp_dir = TempDir::new().unwrap();
    let _guard = lock_output_root(temp_dir.path()).unwrap();

    let second = OpenOptions::new()
      .read(true)
      .write(true)
      .open(temp_dir.path().join(".aptx-codegen.lock"))
      .unwrap();

    assert!(second.try_lock_exclusive().is_err());
  }

  #[test]
  fn lock_output_root_releases_lock_on_drop() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join(".aptx-codegen.lock");

    {
      let _guard = lock_output_root(temp_dir.path()).unwrap();
    }

    let second = OpenOptions::new()
      .read(true)
      .write(true)
      .open(lock_path)
      .unwrap();

    assert!(second.try_lock_exclusive().is_ok());
    second.unlock().unwrap();
  }
}
