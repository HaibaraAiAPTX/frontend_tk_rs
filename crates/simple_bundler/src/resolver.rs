use std::{error::Error, path::PathBuf};

use oxc_resolver::Resolution;

#[derive(Default)]
pub struct Resolver {
    resolver: oxc_resolver::Resolver,
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
                        .unwrap_or_else(|| panic!("dir parent not found: {:?}", dir))
                        .to_path_buf(),
                    entry,
                )
            })
    }
}
