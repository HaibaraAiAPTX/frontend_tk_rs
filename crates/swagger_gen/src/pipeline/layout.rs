use super::model::PlannedFile;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub trait LayoutStrategy {
    fn id(&self) -> &'static str;
    fn apply(&self, files: Vec<PlannedFile>) -> Vec<PlannedFile>;
}

#[derive(Default)]
pub struct IdentityLayout;

impl LayoutStrategy for IdentityLayout {
    fn id(&self) -> &'static str {
        "identity"
    }

    fn apply(&self, files: Vec<PlannedFile>) -> Vec<PlannedFile> {
        files
    }
}

pub fn inject_barrel_indexes(files: Vec<PlannedFile>) -> Vec<PlannedFile> {
    let mut by_path = BTreeMap::<String, PlannedFile>::new();
    for file in files {
        by_path.insert(file.path.clone(), file);
    }

    let source_paths = by_path
        .keys()
        .filter(|path| !path.ends_with("/index.ts") && path.ends_with(".ts"))
        .cloned()
        .collect::<Vec<_>>();
    let roots = ["models", "spec", "functions", "api-query-react", "api-query-vue"];

    let mut target_dirs = BTreeSet::<String>::new();
    for path in &source_paths {
        if let Some(parent) = Path::new(path).parent() {
            let parent_str = parent.to_string_lossy().replace('\\', "/");
            for root in roots {
                if parent_str == root || parent_str.starts_with(&format!("{root}/")) {
                    let mut current = parent_str.clone();
                    loop {
                        target_dirs.insert(current.clone());
                        if current == root {
                            break;
                        }
                        let next = Path::new(&current)
                            .parent()
                            .map(|p| p.to_string_lossy().replace('\\', "/"));
                        match next {
                            Some(value) if !value.is_empty() => current = value,
                            _ => break,
                        }
                    }
                }
            }
        }
    }

    for dir in target_dirs {
        let mut exports = BTreeSet::<String>::new();
        for path in &source_paths {
            let p = Path::new(path);
            let parent = p
                .parent()
                .map(|v| v.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();
            if parent == dir {
                if let Some(stem) = p.file_stem() {
                    let stem = stem.to_string_lossy();
                    if stem != "index" {
                        exports.insert(format!("export * from \"./{}\";", stem));
                    }
                }
                continue;
            }

            let prefix = format!("{dir}/");
            if parent.starts_with(&prefix) {
                let rest = &parent[prefix.len()..];
                if let Some(subdir) = rest.split('/').next() {
                    if !subdir.is_empty() {
                        exports.insert(format!("export * from \"./{}\";", subdir));
                    }
                }
            }
        }

        if exports.is_empty() {
            continue;
        }

        let content = format!("{}\n", exports.into_iter().collect::<Vec<_>>().join("\n"));
        by_path.insert(
            format!("{dir}/index.ts"),
            PlannedFile {
                path: format!("{dir}/index.ts"),
                content,
            },
        );
    }

    by_path.into_values().collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_barrel_indexes_generates_nested_indexes() {
        let files = vec![
            PlannedFile {
                path: "functions/assignment/add.ts".to_string(),
                content: "export const a = 1;\n".to_string(),
            },
            PlannedFile {
                path: "functions/assignment/delete.ts".to_string(),
                content: "export const d = 1;\n".to_string(),
            },
        ];

        let result = inject_barrel_indexes(files);
        let map = result
            .into_iter()
            .map(|f| (f.path, f.content))
            .collect::<BTreeMap<_, _>>();

        assert!(map.contains_key("functions/index.ts"));
        assert!(map.contains_key("functions/assignment/index.ts"));
        assert!(map["functions/index.ts"].contains("export * from \"./assignment\";"));
        assert!(map["functions/assignment/index.ts"].contains("export * from \"./add\";"));
        assert!(map["functions/assignment/index.ts"].contains("export * from \"./delete\";"));
    }
}
