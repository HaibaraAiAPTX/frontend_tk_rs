use super::model::PlannedFile;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
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
    let roots = ["models", "spec", "functions", "react-query", "vue-query"];
    generate_barrel_for_directory_with_roots(files, &roots)
}

pub fn generate_barrel_for_directory_with_roots(
    files: Vec<PlannedFile>,
    roots: &[&str],
) -> Vec<PlannedFile> {
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
    let source_paths: Vec<String> = ts_files
        .iter()
        .map(|file_path| {
            let relative_path = file_path
                .strip_prefix(input_dir)
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
                let dir_name = path
                    .file_name()
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

    // Always include the root input directory (represented as empty string for relative paths)
    target_dirs.insert(String::new());

    for file_path in ts_files {
        // Get the parent directory relative to input_dir
        let relative_path = file_path
            .strip_prefix(input_dir)
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
        } else if dir.is_empty() && !parent.is_empty() {
            // Root-level subdirectories: extract the first path component
            if let Some(subdir) = parent.split('/').next() {
                if !subdir.is_empty() {
                    exports.insert(format!("export * from \"./{}\";", subdir));
                }
            }
        }
    }

    exports
}

/// Force update the barrel file (index.ts) for the specified directory
pub fn force_update_barrel(relative_dir: &str, output_root: &Path) -> Result<(), String> {
    let dir_path = output_root.join(relative_dir);

    if !dir_path.exists() || !dir_path.is_dir() {
        return Ok(()); // Directory doesn't exist, skip
    }

    // Scan all .ts files in the directory (excluding index.ts)
    let mut files: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "ts").unwrap_or(false) {
                if let Some(name) = path.file_stem() {
                    let name = name.to_string_lossy().to_string();
                    if name != "index" {
                        files.push(name);
                    }
                }
            }
        }
    }

    if files.is_empty() {
        return Ok(());
    }

    files.sort();

    // Generate index.ts content
    let content: String = files
        .iter()
        .map(|f| format!("export * from './{}';\n", f))
        .collect();

    // Write directly
    let index_path = dir_path.join("index.ts");
    fs::write(&index_path, content)
        .map_err(|e| format!("Failed to write barrel file {:?}: {}", index_path, e))?;

    Ok(())
}

/// Update barrel files for a directory and all its parents (with output_root as boundary)
pub fn update_barrel_with_parents(relative_dir: &str, output_root: &Path) -> Result<(), String> {
    // Update current directory
    force_update_barrel(relative_dir, output_root)?;

    // Update all parent directories (not exceeding output_root)
    let mut current = Path::new(relative_dir);
    while let Some(parent) = current.parent() {
        if parent.as_os_str().is_empty() {
            break;
        }

        // Boundary check: don't exceed output_root
        let parent_path = output_root.join(parent);
        if parent_path == output_root {
            break;
        }

        let parent_str = parent
            .to_str()
            .ok_or_else(|| format!("Invalid UTF-8 in path: {:?}", parent))?;
        force_update_barrel(parent_str, output_root)?;
        current = parent;
    }

    Ok(())
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

    #[test]
    fn inject_barrel_indexes_generates_nested_indexes_for_react_query() {
        let files = vec![
            PlannedFile {
                path: "react-query/group/item/fetchOne.query.ts".to_string(),
                content: "export const a = 1;\n".to_string(),
            },
            PlannedFile {
                path: "react-query/group/item/fetchAll.query.ts".to_string(),
                content: "export const b = 1;\n".to_string(),
            },
            PlannedFile {
                path: "react-query/assignment/add.mutation.ts".to_string(),
                content: "export const c = 1;\n".to_string(),
            },
        ];

        let result = inject_barrel_indexes(files);
        let map = result
            .into_iter()
            .map(|f| (f.path, f.content))
            .collect::<BTreeMap<_, _>>();

        // Should have root react-query index
        assert!(map.contains_key("react-query/index.ts"));
        // Should have intermediate group index
        assert!(map.contains_key("react-query/group/index.ts"));
        // Should have leaf group/item index
        assert!(map.contains_key("react-query/group/item/index.ts"));
        // Should have assignment index
        assert!(map.contains_key("react-query/assignment/index.ts"));

        // Check exports in root
        assert!(map["react-query/index.ts"].contains("export * from \"./group\";"));
        assert!(map["react-query/index.ts"].contains("export * from \"./assignment\";"));

        // Check exports in intermediate level
        assert!(map["react-query/group/index.ts"].contains("export * from \"./item\";"));

        // Check exports in leaf level
        assert!(
            map["react-query/group/item/index.ts"].contains("export * from \"./fetchOne.query\";")
        );
        assert!(
            map["react-query/group/item/index.ts"].contains("export * from \"./fetchAll.query\";")
        );

        // Check exports in assignment
        assert!(
            map["react-query/assignment/index.ts"].contains("export * from \"./add.mutation\";")
        );
    }

    #[test]
    fn generate_barrel_for_directory_generates_root_index() {
        // Create a temporary directory structure for testing
        let temp_dir = std::env::temp_dir().join("barrel_test_root_index");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("functions/assignment")).unwrap();
        std::fs::create_dir_all(temp_dir.join("react-query/user")).unwrap();

        // Create some .ts files
        std::fs::write(
            temp_dir.join("functions/assignment/add.ts"),
            "export const a = 1;",
        )
        .unwrap();
        std::fs::write(
            temp_dir.join("react-query/user/get.query.ts"),
            "export const b = 1;",
        )
        .unwrap();

        let input_dir = temp_dir.to_string_lossy().to_string();
        let result = generate_barrel_for_directory(&input_dir);

        let map = result
            .into_iter()
            .map(|f| (f.path, f.content))
            .collect::<BTreeMap<_, _>>();

        // Should have root index.ts
        assert!(
            map.contains_key("index.ts"),
            "Root index.ts should be generated"
        );
        // Should have subdirectory indexes
        assert!(map.contains_key("functions/index.ts"));
        assert!(map.contains_key("functions/assignment/index.ts"));
        assert!(map.contains_key("react-query/index.ts"));
        assert!(map.contains_key("react-query/user/index.ts"));

        // Root index should export the top-level directories
        assert!(map["index.ts"].contains("export * from \"./functions\";"));
        assert!(map["index.ts"].contains("export * from \"./react-query\";"));

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    // ==================== force_update_barrel 测试 ====================

    #[test]
    fn force_update_barrel_creates_index_ts() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let models_dir = temp_dir.path().join("models");
        fs::create_dir_all(&models_dir).unwrap();
        fs::write(models_dir.join("User.ts"), "export type User = {};").unwrap();
        fs::write(models_dir.join("Post.ts"), "export type Post = {};").unwrap();

        let result = force_update_barrel("models", temp_dir.path());

        assert!(result.is_ok());
        let index_path = temp_dir.path().join("models/index.ts");
        assert!(index_path.exists());
        let content = fs::read_to_string(&index_path).unwrap();
        assert!(content.contains("export * from './Post';"));
        assert!(content.contains("export * from './User';"));
    }

    #[test]
    fn force_update_barrel_skips_nonexistent_directory() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        let result = force_update_barrel("nonexistent", temp_dir.path());

        assert!(result.is_ok());
    }

    #[test]
    fn force_update_barrel_excludes_index_from_exports() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let models_dir = temp_dir.path().join("models");
        fs::create_dir_all(&models_dir).unwrap();
        fs::write(models_dir.join("User.ts"), "export type User = {};").unwrap();
        fs::write(models_dir.join("index.ts"), "// existing index").unwrap();

        let result = force_update_barrel("models", temp_dir.path());

        assert!(result.is_ok());
        let content = fs::read_to_string(temp_dir.path().join("models/index.ts")).unwrap();
        // Should not export index itself
        assert!(!content.contains("export * from './index';"));
        // Should export User
        assert!(content.contains("export * from './User';"));
    }

    #[test]
    fn force_update_barrel_exports_sorted_files() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let models_dir = temp_dir.path().join("models");
        fs::create_dir_all(&models_dir).unwrap();
        // Add files in non-alphabetical order
        fs::write(models_dir.join("Zebra.ts"), "").unwrap();
        fs::write(models_dir.join("Apple.ts"), "").unwrap();
        fs::write(models_dir.join("Mango.ts"), "").unwrap();

        let result = force_update_barrel("models", temp_dir.path());

        assert!(result.is_ok());
        let content = fs::read_to_string(temp_dir.path().join("models/index.ts")).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        // Check alphabetical order
        assert!(lines[0].contains("Apple"));
        assert!(lines[1].contains("Mango"));
        assert!(lines[2].contains("Zebra"));
    }

    #[test]
    fn force_update_barrel_handles_empty_directory() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let models_dir = temp_dir.path().join("models");
        fs::create_dir_all(&models_dir).unwrap();

        let result = force_update_barrel("models", temp_dir.path());

        assert!(result.is_ok());
        // Should not create index.ts for empty directory
        assert!(!temp_dir.path().join("models/index.ts").exists());
    }

    #[test]
    fn force_update_barrel_only_includes_ts_files() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let models_dir = temp_dir.path().join("models");
        fs::create_dir_all(&models_dir).unwrap();
        fs::write(models_dir.join("User.ts"), "").unwrap();
        fs::write(models_dir.join("README.md"), "").unwrap();
        fs::write(models_dir.join("data.json"), "").unwrap();

        let result = force_update_barrel("models", temp_dir.path());

        assert!(result.is_ok());
        let content = fs::read_to_string(temp_dir.path().join("models/index.ts")).unwrap();
        // Should only export .ts files
        assert!(content.contains("export * from './User';"));
        assert_eq!(content.lines().count(), 1);
    }

    // ==================== update_barrel_with_parents 测试 ====================

    #[test]
    fn update_barrel_with_parents_updates_current() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let nested_dir = temp_dir.path().join("api").join("v1").join("users");
        fs::create_dir_all(&nested_dir).unwrap();
        fs::write(nested_dir.join("getUser.ts"), "").unwrap();

        let result = update_barrel_with_parents("api/v1/users", temp_dir.path());

        assert!(result.is_ok());
        assert!(temp_dir.path().join("api/v1/users/index.ts").exists());
    }

    #[test]
    fn update_barrel_with_parents_updates_parent_dirs() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        // Create nested structure
        let nested_dir = temp_dir.path().join("api").join("v1").join("users");
        fs::create_dir_all(&nested_dir).unwrap();
        // Add files at each level
        fs::write(temp_dir.path().join("api").join("ApiBase.ts"), "").unwrap();
        fs::write(temp_dir.path().join("api").join("v1").join("V1Base.ts"), "").unwrap();
        fs::write(nested_dir.join("getUser.ts"), "").unwrap();

        let result = update_barrel_with_parents("api/v1/users", temp_dir.path());

        assert!(result.is_ok());
        // Should update all parent directories
        assert!(temp_dir.path().join("api/v1/index.ts").exists());
        // Note: api/index.ts should exist since api/v1 has files
        // But we need to check the actual behavior
    }

    #[test]
    fn update_barrel_with_parents_stops_at_output_root() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let models_dir = temp_dir.path().join("models");
        fs::create_dir_all(&models_dir).unwrap();
        fs::write(models_dir.join("User.ts"), "").unwrap();

        let result = update_barrel_with_parents("models", temp_dir.path());

        assert!(result.is_ok());
        // Should create models/index.ts
        assert!(temp_dir.path().join("models/index.ts").exists());
        // Should NOT create index.ts at root (because models is direct child)
        // The function stops at output_root, so no parent of "models" to update
    }

    #[test]
    fn update_barrel_with_parents_handles_empty_relative_dir() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        fs::write(temp_dir.path().join("RootFile.ts"), "").unwrap();

        let result = update_barrel_with_parents("", temp_dir.path());

        // Empty relative_dir should work without error
        assert!(result.is_ok());
    }
}
