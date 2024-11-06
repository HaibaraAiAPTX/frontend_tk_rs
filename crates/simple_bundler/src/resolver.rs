use std::{error::Error, path::PathBuf};

use oxc_resolver::{Resolution, ResolveOptions};

pub struct Resolver {
    resolver: oxc_resolver::Resolver,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            resolver: oxc_resolver::Resolver::new(ResolveOptions {
                condition_names: vec!["import".into()],
                main_fields: vec!["module".into(), "main".into()],
                extensions: vec![".js".into(), ".json".into(), ".ts".into()],
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
                    "resolve failed, try parent dir {} specific {}",
                    dir.display(),
                    entry.display()
                );
                self.resolve(
                    &dir.parent()
                        .expect(&format!("dir parent not found: {:?}", dir))
                        .to_path_buf(),
                    entry,
                )
            })
            .map_err(|e| e.into())
    }
}
