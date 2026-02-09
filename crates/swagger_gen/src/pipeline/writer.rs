use std::{
    fs,
    path::{Path, PathBuf},
};

use super::model::{PlannedFile, WritePlan};

pub trait Writer {
    fn id(&self) -> &'static str;
    fn write(&self, files: Vec<PlannedFile>) -> Result<WritePlan, String>;
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

impl FileSystemWriter {
    pub fn new(output_root: impl AsRef<Path>) -> Self {
        Self {
            output_root: output_root.as_ref().to_path_buf(),
        }
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

            let tmp_path = full_path.with_extension("tmp");
            fs::write(&tmp_path, &file.content).map_err(|err| err.to_string())?;
            if full_path.exists() {
                fs::remove_file(&full_path).map_err(|err| err.to_string())?;
            }
            fs::rename(&tmp_path, &full_path).map_err(|err| err.to_string())?;
            files_to_write.push(file);
        }
        Ok(WritePlan {
            files_to_write,
            skipped_files,
        })
    }
}
