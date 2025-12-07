use std::{error::Error, path::PathBuf};

use oxc_resolver::{Resolution, ResolveOptions};

pub struct Resolver {
    resolver: oxc_resolver::Resolver,
}

impl Default for Resolver {
    fn default() -> Self {
        Self {
            resolver: oxc_resolver::Resolver::new(ResolveOptions {
                condition_names: vec!["import".into()],
                main_fields: vec!["module".into(), "main".into()],
                extensions: vec![".js".into(), ".json".into(), ".ts".into(), ".node".into()],
                ..Default::default()
            }),
        }
    }
}

impl Resolver {
    pub fn resolve(&self, dir: &PathBuf, entry: &PathBuf) -> Result<Resolution, Box<dyn Error>> {
        let specifier = entry.to_str().unwrap().to_string();
        self.resolver
            .resolve(dir.as_path(), &specifier)
            .or_else(|_| {
                println!(
                    "resolve failed, try parent dir:{} specific:{}",
                    dir.display(),
                    entry.display()
                );
                self.resolve(
                    &dir.parent()
                        .unwrap_or_else(|| panic!("dir parent not found: {:?}", dir))
                        .to_path_buf(),
                    entry,
                )
            })
    }
}
