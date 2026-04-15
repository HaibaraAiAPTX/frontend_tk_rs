use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use super::model::{PlannedFile, WritePlan};

pub trait Writer {
    fn id(&self) -> &'static str;
    fn write(&self, files: Vec<PlannedFile>) -> Result<WritePlan, String>;
    fn output_root(&self) -> Option<&Path> {
        None
    }
}

#[derive(Default)]
pub struct DryRunWriter;

impl Writer for DryRunWriter {
    fn id(&self) -> &'static str {
        "dry-run"
    }

    fn write(&self, files: Vec<PlannedFile>) -> Result<WritePlan, String> {
        Ok(WritePlan {
            files_to_write: files,
            skipped_files: 0,
        })
    }
}

pub struct FileSystemWriter {
    output_root: PathBuf,
}

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

impl FileSystemWriter {
    pub fn new(output_root: impl AsRef<Path>) -> Self {
        Self {
            output_root: output_root.as_ref().to_path_buf(),
        }
    }

    fn tmp_path_for(full_path: &Path) -> PathBuf {
        let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let pid = std::process::id();
        let file_name = full_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "codegen".to_string());

        full_path.with_file_name(format!("{file_name}.{pid}.{timestamp}.{counter}.tmp"))
    }
}

impl Writer for FileSystemWriter {
    fn id(&self) -> &'static str {
        "fs"
    }

    fn write(&self, files: Vec<PlannedFile>) -> Result<WritePlan, String> {
        let mut files_to_write = Vec::new();
        let mut skipped_files = 0usize;
        for file in files {
            let full_path = self.output_root.join(&file.path);
            if let Ok(existing) = fs::read_to_string(&full_path) {
                if existing == file.content {
                    skipped_files += 1;
                    continue;
                }
            }

            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).map_err(|err| err.to_string())?;
            }

            let tmp_path = Self::tmp_path_for(&full_path);
            fs::write(&tmp_path, &file.content).map_err(|err| err.to_string())?;
            if let Err(err) = fs::remove_file(&full_path) {
                if err.kind() != std::io::ErrorKind::NotFound {
                    let _ = fs::remove_file(&tmp_path);
                    return Err(err.to_string());
                }
            }
            if let Err(err) = fs::rename(&tmp_path, &full_path) {
                let _ = fs::remove_file(&tmp_path);
                return Err(err.to_string());
            }
            files_to_write.push(file);
        }
        Ok(WritePlan {
            files_to_write,
            skipped_files,
        })
    }

    fn output_root(&self) -> Option<&Path> {
        Some(&self.output_root)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::FileSystemWriter;

    #[test]
    fn tmp_path_for_is_unique_per_call() {
        let target = Path::new("models/User.ts");

        let first = FileSystemWriter::tmp_path_for(target);
        let second = FileSystemWriter::tmp_path_for(target);

        assert_ne!(first, second);
        assert_eq!(first.parent(), target.parent());
        assert_eq!(second.parent(), target.parent());
    }
}
