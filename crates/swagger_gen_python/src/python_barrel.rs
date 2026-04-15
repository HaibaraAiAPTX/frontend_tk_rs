use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use swagger_gen::pipeline::PlannedFile;

pub fn generate_python_package_inits_for_directory(
    input_dir: &str,
) -> Result<Vec<PlannedFile>, String> {
    let python_files = collect_python_files(input_dir);
    let mut leaf_exports = Vec::new();

    for file_path in python_files {
        let relative_path = file_path
            .strip_prefix(input_dir)
            .map(|p| p.trim_start_matches('/').trim_start_matches('\\'))
            .unwrap_or(file_path.as_str())
            .replace('\\', "/");
        let content = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;
        let fallback_symbol = Path::new(&relative_path)
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap_or_default();

        for symbol in extract_python_module_exports(&content, &fallback_symbol) {
            leaf_exports.push((relative_path.clone(), symbol));
        }
    }

    Ok(build_python_package_inits(&leaf_exports))
}

fn build_python_package_inits(leaf_exports: &[(String, String)]) -> Vec<PlannedFile> {
    let mut dir_exports: BTreeMap<String, BTreeSet<(String, String)>> = BTreeMap::new();

    for (module_path, symbol) in leaf_exports {
        let module_without_ext = module_path.trim_end_matches(".py").replace('/', ".");
        let module_dir = Path::new(module_path)
            .parent()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();

        let mut current = module_dir.clone();
        loop {
            let current_prefix = current.replace('/', ".");
            let relative_module = if current_prefix.is_empty() {
                module_without_ext.clone()
            } else {
                module_without_ext
                    .strip_prefix(&(current_prefix + "."))
                    .unwrap_or(module_without_ext.as_str())
                    .to_string()
            };

            dir_exports
                .entry(current.clone())
                .or_default()
                .insert((format!(".{relative_module}"), symbol.clone()));

            if current.is_empty() {
                break;
            }

            current = Path::new(&current)
                .parent()
                .map(|path| path.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();
        }
    }

    dir_exports
        .into_iter()
        .map(|(dir, exports)| {
            let import_lines = exports
                .iter()
                .map(|(module_path, symbol)| format!("from {module_path} import {symbol}"))
                .collect::<Vec<_>>();
            let all_exports = exports
                .iter()
                .map(|(_, symbol)| format!("\"{symbol}\""))
                .collect::<Vec<_>>();
            let path = if dir.is_empty() {
                "__init__.py".to_string()
            } else {
                format!("{dir}/__init__.py")
            };

            PlannedFile {
                path,
                content: format!(
                    "{}\n\n__all__ = [{}]\n",
                    import_lines.join("\n"),
                    all_exports.join(", ")
                ),
            }
        })
        .collect()
}

fn collect_python_files(input_dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    let dir_path = Path::new(input_dir);

    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let is_python_file = path.extension().is_some_and(|ext| ext == "py");
                let is_init = path.file_name().is_some_and(|name| name == "__init__.py");
                if is_python_file && !is_init {
                    files.push(path.to_string_lossy().to_string());
                }
            } else if path.is_dir() {
                let dir_name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_default();
                if !dir_name.starts_with('.') && dir_name != "__pycache__" {
                    files.extend(collect_python_files(&path.to_string_lossy()));
                }
            }
        }
    }

    files
}

fn extract_python_module_exports(content: &str, fallback_symbol: &str) -> Vec<String> {
    let mut exports = BTreeSet::new();

    for line in content.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }

        if let Some(symbol) = line
            .strip_prefix("class ")
            .and_then(|rest| rest.split(['(', ':']).next())
            .map(str::trim)
            .filter(|name| is_public_python_identifier(name))
        {
            exports.insert(symbol.to_string());
            continue;
        }

        if let Some(symbol) = line
            .strip_prefix("def ")
            .and_then(|rest| rest.split('(').next())
            .map(str::trim)
            .filter(|name| is_public_python_identifier(name))
        {
            exports.insert(symbol.to_string());
        }
    }

    if exports.is_empty() && is_public_python_identifier(fallback_symbol) {
        exports.insert(fallback_symbol.to_string());
    }

    exports.into_iter().collect()
}

fn is_public_python_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first == '_' || !(first.is_ascii_alphabetic() || first == '_') => false,
        Some(_) => chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_'),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::generate_python_package_inits_for_directory;
    use std::collections::BTreeMap;
    use std::fs;

    #[test]
    fn generate_python_package_inits_for_directory_generates_root_and_nested_inits() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::create_dir_all(root.join("functions/action_authority")).unwrap();
        fs::create_dir_all(root.join("spec/action_authority")).unwrap();
        fs::create_dir_all(root.join("models")).unwrap();

        fs::write(
            root.join("functions/action_authority/add.py"),
            "def action_authority_add() -> None:\n    pass\n",
        )
        .unwrap();
        fs::write(
            root.join("spec/action_authority/add_spec.py"),
            "def build_action_authority_add_spec() -> None:\n    pass\n",
        )
        .unwrap();
        fs::write(root.join("models/User.py"), "class User:\n    pass\n").unwrap();

        let planned_files =
            generate_python_package_inits_for_directory(&root.to_string_lossy()).unwrap();
        let files = planned_files
            .into_iter()
            .map(|file| (file.path, file.content))
            .collect::<BTreeMap<_, _>>();

        assert!(files.contains_key("__init__.py"));
        assert!(files.contains_key("functions/__init__.py"));
        assert!(files.contains_key("functions/action_authority/__init__.py"));
        assert!(files.contains_key("spec/__init__.py"));
        assert!(files.contains_key("models/__init__.py"));

        assert!(
            files["__init__.py"]
                .contains("from .functions.action_authority.add import action_authority_add")
        );
        assert!(files["__init__.py"].contains("from .models.User import User"));
        assert!(
            files["functions/__init__.py"]
                .contains("from .action_authority.add import action_authority_add")
        );
        assert!(
            files["spec/action_authority/__init__.py"]
                .contains("from .add_spec import build_action_authority_add_spec")
        );
    }
}
