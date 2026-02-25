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
    let roots = ["models", "spec", "functions", "api-query-react", "api-query-vue"];
    generate_barrel_for_directory_with_roots(files, &roots)
}

pub fn generate_barrel_for_directory_with_roots(files: Vec<PlannedFile>, roots: &[&str]) -> Vec<PlannedFile> {
    let mut by_path = BTreeMap::<String, PlannedFile>::new();
    for file in files {
        by_path.insert(file.path.clone(), file);
    }

    let source_paths = by_path
        .keys()
        .filter(|path| !path.ends_with("/index.ts") && path.ends_with(".ts"))
        .cloned()
        .collect::<Vec<_>>();

    let mut target_dirs = BTreeSet::<String>::new();
    for path in &source_paths {
        if let Some(parent) = Path::new(path).parent() {
            let parent_str = parent.to_string_lossy().replace('\\', "/");
            for root in roots {
                if parent_str == *root || parent_str.starts_with(&format!("{root}/")) {
                    let mut current = parent_str.clone();
                    loop {
                        target_dirs.insert(current.clone());
                        if current == *root {
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
        let exports = collect_exports_for_dir(&source_paths, &dir);
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

/// Generate barrel index.ts files for all subdirectories in the given input directory.
/// Scans all subdirectories and generates:
/// - index.ts in each subdirectory containing .ts files (exports all .ts files)
/// - index.ts in the input directory (exports all subdirectories)
///
/// Returns only the generated barrel index.ts files, not the source files.
pub fn generate_barrel_for_directory(input_dir: &str) -> Vec<PlannedFile> {
    // Collect all .ts files from the input directory recursively
    let ts_files = collect_ts_files(input_dir);

    // Get the list of source file paths
    let source_paths: Vec<String> = ts_files.iter()
        .map(|file_path| {
            let relative_path = file_path.strip_prefix(input_dir)
                .map(|p| p.trim_start_matches('/').trim_start_matches('\\'))
                .unwrap_or(file_path);
            relative_path.replace('\\', "/")
        })
        .collect();

    // Generate barrel files for the input directory and all subdirectories
    let target_dirs = collect_target_dirs(&ts_files, input_dir);

    let mut barrel_files = Vec::new();

    for dir in target_dirs {
        let exports = collect_exports_for_dir(&source_paths, &dir);

        if exports.is_empty() {
            continue;
        }

        let content = format!("{}\n", exports.into_iter().collect::<Vec<_>>().join("\n"));
        let index_path = if dir.is_empty() {
            "index.ts".to_string()
        } else {
            format!("{dir}/index.ts")
        };

        barrel_files.push(PlannedFile {
            path: index_path,
            content,
        });
    }

    barrel_files
}

/// Collect all .ts files recursively from a directory
fn collect_ts_files(input_dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    let dir_path = Path::new(input_dir);

    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "ts" {
                        files.push(path.to_string_lossy().to_string());
                    }
                }
            } else if path.is_dir() {
                // Skip node_modules and other hidden directories
                let dir_name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                if !dir_name.starts_with('.') && dir_name != "node_modules" {
                    files.extend(collect_ts_files(&path.to_string_lossy()));
                }
            }
        }
    }

    files
}

/// Collect all target directories that need barrel files
fn collect_target_dirs(ts_files: &[String], input_dir: &str) -> BTreeSet<String> {
    let mut target_dirs = BTreeSet::<String>::new();

    // Always include the root input directory
    target_dirs.insert(input_dir.to_string());

    for file_path in ts_files {
        // Get the parent directory relative to input_dir
        let relative_path = file_path.strip_prefix(input_dir)
            .map(|p| p.trim_start_matches('/').trim_start_matches('\\'))
            .unwrap_or(file_path);

        let path = Path::new(relative_path);
        if let Some(parent) = path.parent() {
            let parent_str = parent.to_string_lossy().replace('\\', "/");
            if !parent_str.is_empty() && parent_str != "." {
                // Add all parent directories up to the input directory
                let mut current = parent_str.clone();
                loop {
                    target_dirs.insert(current.clone());
                    if current.is_empty() {
                        break;
                    }
                    let next = Path::new(&current)
                        .parent()
                        .map(|p| p.to_string_lossy().replace('\\', "/"));
                    match next {
                        Some(value) if !value.is_empty() && value != "." => current = value,
                        _ => break,
                    }
                }
            }
        }
    }

    target_dirs
}

/// Collect exports for a specific directory
fn collect_exports_for_dir(source_paths: &[String], dir: &str) -> BTreeSet<String> {
    let mut exports = BTreeSet::<String>::new();

    for path in source_paths {
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

        let prefix = if dir.is_empty() {
            "".to_string()
        } else {
            format!("{dir}/")
        };

        if !prefix.is_empty() && parent.starts_with(&prefix) {
            let rest = &parent[prefix.len()..];
            if let Some(subdir) = rest.split('/').next() {
                if !subdir.is_empty() {
                    exports.insert(format!("export * from \"./{}\";", subdir));
                }
            }
        } else if dir.is_empty() && !parent.is_empty() && !parent.contains('/') {
            // Root-level subdirectories
            exports.insert(format!("export * from \"./{}\";", parent));
        }
    }

    exports
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
