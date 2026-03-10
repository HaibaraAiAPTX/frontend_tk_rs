use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Manifest 版本
pub const MANIFEST_VERSION: &str = "1.0";

/// 生成清单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// 清单版本
    pub version: String,
    /// 生成时间
    pub generated_at: DateTime<Utc>,
    /// OpenAPI 规范的 hash
    pub openapi_hash: String,
    /// OpenAPI 版本
    pub openapi_version: String,
    /// 生成器版本
    pub generator_version: String,
    /// 各生成器的产物记录
    pub generators: HashMap<String, HashMap<String, String>>,
}

impl Manifest {
    /// 创建新的清单
    pub fn new(openapi_hash: String, openapi_version: String, generator_version: String) -> Self {
        Self {
            version: MANIFEST_VERSION.to_string(),
            generated_at: Utc::now(),
            openapi_hash,
            openapi_version,
            generator_version,
            generators: HashMap::new(),
        }
    }

    /// 从文件加载
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        if !path.exists() {
            return Err("Manifest file not found".to_string());
        }
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse manifest: {}", e))
    }

    /// 保存到文件
    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create manifest directory: {}", e))?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write manifest: {}", e))
    }

    /// 获取指定生成器的条目
    pub fn get_generator_entries(&self, generator_id: &str) -> Option<&HashMap<String, String>> {
        self.generators.get(generator_id)
    }

    /// 更新指定生成器的条目
    pub fn update_generator(&mut self, generator_id: String, entries: HashMap<String, String>) {
        self.generated_at = Utc::now();
        self.generators.insert(generator_id, entries);
    }
}

impl Default for Manifest {
    fn default() -> Self {
        Self::new(
            String::new(),
            String::new(),
            env!("CARGO_PKG_VERSION").to_string(),
        )
    }
}

/// 变更对比结果
#[derive(Debug, Clone)]
pub struct ManifestDiff {
    /// 生成器 ID
    pub generator_id: String,
    /// 新增的条目 (name, path)
    pub added: Vec<(String, String)>,
    /// 删除的条目 (name, path)
    pub deleted: Vec<(String, String)>,
    /// 未变更的条目名称
    pub unchanged: Vec<String>,
}

impl ManifestDiff {
    /// 是否有任何变更
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.deleted.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ==================== Manifest 结构测试 ====================

    #[test]
    fn manifest_new_creates_valid_instance() {
        let manifest = Manifest::new(
            "hash123".to_string(),
            "3.0.0".to_string(),
            "1.0.0".to_string(),
        );

        assert_eq!(manifest.version, MANIFEST_VERSION);
        assert_eq!(manifest.openapi_hash, "hash123");
        assert_eq!(manifest.openapi_version, "3.0.0");
        assert_eq!(manifest.generator_version, "1.0.0");
        assert!(manifest.generators.is_empty());
    }

    #[test]
    fn manifest_default_creates_empty_instance() {
        let manifest = Manifest::default();

        assert_eq!(manifest.version, MANIFEST_VERSION);
        assert!(manifest.openapi_hash.is_empty());
        assert!(manifest.openapi_version.is_empty());
        assert_eq!(manifest.generator_version, env!("CARGO_PKG_VERSION"));
        assert!(manifest.generators.is_empty());
    }

    #[test]
    fn manifest_save_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested").join("dir").join("manifest.json");

        let manifest = Manifest::default();
        let result = manifest.save(&nested_path);

        assert!(result.is_ok());
        assert!(nested_path.exists());
    }

    #[test]
    fn manifest_load_parses_valid_json() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.json");

        let original = Manifest::new(
            "hash123".to_string(),
            "3.0.0".to_string(),
            "1.0.0".to_string(),
        );
        original.save(&manifest_path).unwrap();

        let loaded = Manifest::load(&manifest_path);

        assert!(loaded.is_ok());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.openapi_hash, "hash123");
        assert_eq!(loaded.openapi_version, "3.0.0");
        assert_eq!(loaded.generator_version, "1.0.0");
    }

    #[test]
    fn manifest_load_fails_on_missing_file() {
        let result = Manifest::load(std::path::Path::new("/nonexistent/path/manifest.json"));

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn manifest_load_fails_on_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.json");

        std::fs::write(&manifest_path, "{ invalid json }").unwrap();

        let result = Manifest::load(&manifest_path);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("parse"));
    }

    #[test]
    fn manifest_save_and_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.json");

        let mut original = Manifest::new(
            "hash123".to_string(),
            "3.0.0".to_string(),
            "1.0.0".to_string(),
        );
        let mut entries = HashMap::new();
        entries.insert("Model1".to_string(), "model1.ts".to_string());
        entries.insert("Model2".to_string(), "model2.ts".to_string());
        original.update_generator("models".to_string(), entries);

        original.save(&manifest_path).unwrap();
        let loaded = Manifest::load(&manifest_path).unwrap();

        assert_eq!(loaded.version, original.version);
        assert_eq!(loaded.openapi_hash, original.openapi_hash);
        assert_eq!(loaded.openapi_version, original.openapi_version);
        assert_eq!(loaded.generator_version, original.generator_version);
        assert_eq!(loaded.generators.len(), 1);
        assert!(loaded.generators.contains_key("models"));
    }

    // ==================== Manifest 方法测试 ====================

    #[test]
    fn manifest_get_generator_entries_returns_some() {
        let mut manifest = Manifest::default();
        let mut entries = HashMap::new();
        entries.insert("Model1".to_string(), "model1.ts".to_string());
        manifest.update_generator("models".to_string(), entries);

        let result = manifest.get_generator_entries("models");

        assert!(result.is_some());
        assert!(result.unwrap().contains_key("Model1"));
    }

    #[test]
    fn manifest_get_generator_entries_returns_none() {
        let manifest = Manifest::default();

        let result = manifest.get_generator_entries("nonexistent");

        assert!(result.is_none());
    }

    #[test]
    fn manifest_update_generator_inserts_new() {
        let mut manifest = Manifest::default();
        let entries = HashMap::from([
            ("Model1".to_string(), "model1.ts".to_string()),
        ]);

        manifest.update_generator("models".to_string(), entries);

        assert_eq!(manifest.generators.len(), 1);
        assert!(manifest.generators.contains_key("models"));
    }

    #[test]
    fn manifest_update_generator_replaces_existing() {
        let mut manifest = Manifest::default();

        let entries1 = HashMap::from([
            ("Model1".to_string(), "model1.ts".to_string()),
        ]);
        manifest.update_generator("models".to_string(), entries1);

        let entries2 = HashMap::from([
            ("Model2".to_string(), "model2.ts".to_string()),
        ]);
        manifest.update_generator("models".to_string(), entries2);

        assert_eq!(manifest.generators.len(), 1);
        let generator_entries = manifest.get_generator_entries("models").unwrap();
        assert!(!generator_entries.contains_key("Model1"));
        assert!(generator_entries.contains_key("Model2"));
    }

    #[test]
    fn manifest_update_generator_updates_timestamp() {
        let mut manifest = Manifest::default();
        let original_time = manifest.generated_at;

        // Wait a tiny bit to ensure time difference
        std::thread::sleep(std::time::Duration::from_millis(1));

        let entries = HashMap::new();
        manifest.update_generator("models".to_string(), entries);

        assert!(manifest.generated_at > original_time);
    }

    // ==================== ManifestDiff 测试 ====================

    #[test]
    fn manifest_diff_has_changes_true_when_added() {
        let diff = ManifestDiff {
            generator_id: "models".to_string(),
            added: vec![("Model1".to_string(), "model1.ts".to_string())],
            deleted: vec![],
            unchanged: vec![],
        };

        assert!(diff.has_changes());
    }

    #[test]
    fn manifest_diff_has_changes_true_when_deleted() {
        let diff = ManifestDiff {
            generator_id: "models".to_string(),
            added: vec![],
            deleted: vec![("Model1".to_string(), "model1.ts".to_string())],
            unchanged: vec![],
        };

        assert!(diff.has_changes());
    }

    #[test]
    fn manifest_diff_has_changes_false_when_empty() {
        let diff = ManifestDiff {
            generator_id: "models".to_string(),
            added: vec![],
            deleted: vec![],
            unchanged: vec!["Model1".to_string()],
        };

        assert!(!diff.has_changes());
    }
}
