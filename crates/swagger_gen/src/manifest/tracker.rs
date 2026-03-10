use super::{Manifest, ManifestDiff};
use std::collections::HashMap;
use std::path::Path;

/// 文件追踪器，用于记录生成过程中的文件
pub struct ManifestTracker {
    generator_id: String,
    entries: HashMap<String, String>,
}

impl ManifestTracker {
    /// 创建新的追踪器
    pub fn new(generator_id: impl Into<String>) -> Self {
        Self {
            generator_id: generator_id.into(),
            entries: HashMap::new(),
        }
    }

    /// 记录生成的文件
    pub fn track(&mut self, name: impl Into<String>, path: impl Into<String>) {
        self.entries.insert(name.into(), path.into());
    }

    /// 批量记录
    pub fn track_batch(&mut self, entries: Vec<(String, String)>) {
        for (name, path) in entries {
            self.entries.insert(name, path);
        }
    }

    /// 完成追踪，对比现有 manifest 并返回差异
    pub fn finish(self, manifest_path: &Path) -> ManifestDiff {
        let existing_entries = Manifest::load(manifest_path)
            .ok()
            .and_then(|m| m.get_generator_entries(&self.generator_id).cloned())
            .unwrap_or_default();

        let mut added = Vec::new();
        let mut deleted = Vec::new();
        let mut unchanged = Vec::new();

        // 找出新增和未变更的
        for (name, path) in &self.entries {
            if existing_entries.contains_key(name) {
                unchanged.push(name.clone());
            } else {
                added.push((name.clone(), path.clone()));
            }
        }

        // 找出被删除的
        for (name, path) in &existing_entries {
            if !self.entries.contains_key(name) {
                deleted.push((name.clone(), path.clone()));
            }
        }

        ManifestDiff {
            generator_id: self.generator_id,
            added,
            deleted,
            unchanged,
        }
    }

    /// 获取当前记录的条目
    pub fn entries(&self) -> &HashMap<String, String> {
        &self.entries
    }

    /// 获取生成器 ID
    pub fn generator_id(&self) -> &str {
        &self.generator_id
    }
}

/// 更新 manifest 文件
pub fn update_manifest(
    manifest_path: &Path,
    generator_id: String,
    entries: HashMap<String, String>,
    openapi_hash: &str,
    openapi_version: &str,
) -> Result<Manifest, String> {
    let mut manifest = Manifest::load(manifest_path).unwrap_or_else(|_| {
        Manifest::new(
            openapi_hash.to_string(),
            openapi_version.to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        )
    });

    // 更新 hash 和版本（如果有变化）
    if !openapi_hash.is_empty() {
        manifest.openapi_hash = openapi_hash.to_string();
    }
    if !openapi_version.is_empty() {
        manifest.openapi_version = openapi_version.to_string();
    }

    manifest.update_generator(generator_id, entries);
    manifest.save(manifest_path)?;

    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Manifest;
    use tempfile::TempDir;

    // ==================== ManifestTracker 测试 ====================

    #[test]
    fn tracker_new_creates_empty_instance() {
        let tracker = ManifestTracker::new("models");

        assert_eq!(tracker.generator_id(), "models");
        assert!(tracker.entries().is_empty());
    }

    #[test]
    fn tracker_track_adds_single_entry() {
        let mut tracker = ManifestTracker::new("models");

        tracker.track("Model1", "model1.ts");

        assert_eq!(tracker.entries().len(), 1);
        assert_eq!(tracker.entries().get("Model1"), Some(&"model1.ts".to_string()));
    }

    #[test]
    fn tracker_track_overwrites_existing() {
        let mut tracker = ManifestTracker::new("models");

        tracker.track("Model1", "model1.ts");
        tracker.track("Model1", "updated_model1.ts");

        assert_eq!(tracker.entries().len(), 1);
        assert_eq!(tracker.entries().get("Model1"), Some(&"updated_model1.ts".to_string()));
    }

    #[test]
    fn tracker_track_batch_adds_multiple_entries() {
        let mut tracker = ManifestTracker::new("models");

        tracker.track_batch(vec![
            ("Model1".to_string(), "model1.ts".to_string()),
            ("Model2".to_string(), "model2.ts".to_string()),
        ]);

        assert_eq!(tracker.entries().len(), 2);
        assert!(tracker.entries().contains_key("Model1"));
        assert!(tracker.entries().contains_key("Model2"));
    }

    #[test]
    fn tracker_entries_returns_reference() {
        let mut tracker = ManifestTracker::new("models");
        tracker.track("Model1", "model1.ts");

        let entries = tracker.entries();

        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn tracker_generator_id_returns_reference() {
        let tracker = ManifestTracker::new("models");

        assert_eq!(tracker.generator_id(), "models");
    }

    // ==================== finish() 方法测试 ====================

    #[test]
    fn tracker_finish_detects_added_entries() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.json");

        let mut tracker = ManifestTracker::new("models");
        tracker.track("Model1", "model1.ts");
        tracker.track("Model2", "model2.ts");

        let diff = tracker.finish(&manifest_path);

        assert_eq!(diff.added.len(), 2);
        assert!(diff.added.iter().any(|(n, _)| n == "Model1"));
        assert!(diff.added.iter().any(|(n, _)| n == "Model2"));
        assert!(diff.deleted.is_empty());
    }

    #[test]
    fn tracker_finish_detects_deleted_entries() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.json");

        // 创建现有 manifest
        let mut manifest = Manifest::default();
        let entries = HashMap::from([
            ("OldModel1".to_string(), "old_model1.ts".to_string()),
            ("OldModel2".to_string(), "old_model2.ts".to_string()),
        ]);
        manifest.update_generator("models".to_string(), entries);
        manifest.save(&manifest_path).unwrap();

        // 创建只包含一个新模型的 tracker
        let mut tracker = ManifestTracker::new("models");
        tracker.track("Model1", "model1.ts");

        let diff = tracker.finish(&manifest_path);

        assert_eq!(diff.deleted.len(), 2);
        assert!(diff.deleted.iter().any(|(n, _)| n == "OldModel1"));
        assert!(diff.deleted.iter().any(|(n, _)| n == "OldModel2"));
    }

    #[test]
    fn tracker_finish_detects_unchanged_entries() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.json");

        // 创建现有 manifest
        let mut manifest = Manifest::default();
        let entries = HashMap::from([
            ("Model1".to_string(), "model1.ts".to_string()),
        ]);
        manifest.update_generator("models".to_string(), entries);
        manifest.save(&manifest_path).unwrap();

        // 创建包含相同模型的 tracker
        let mut tracker = ManifestTracker::new("models");
        tracker.track("Model1", "model1.ts");
        tracker.track("Model2", "model2.ts");

        let diff = tracker.finish(&manifest_path);

        assert!(diff.unchanged.contains(&"Model1".to_string()));
        assert!(!diff.unchanged.contains(&"Model2".to_string()));
    }

    #[test]
    fn tracker_finish_handles_new_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.json");

        let mut tracker = ManifestTracker::new("models");
        tracker.track("Model1", "model1.ts");

        // 不创建 manifest 文件
        let diff = tracker.finish(&manifest_path);

        // 所有条目应该是 added
        assert_eq!(diff.added.len(), 1);
        assert!(diff.deleted.is_empty());
    }

    #[test]
    fn tracker_finish_handles_empty_tracker() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.json");

        // 创建现有 manifest
        let mut manifest = Manifest::default();
        let entries = HashMap::from([
            ("OldModel".to_string(), "old_model.ts".to_string()),
        ]);
        manifest.update_generator("models".to_string(), entries);
        manifest.save(&manifest_path).unwrap();

        // 空 tracker
        let tracker = ManifestTracker::new("models");
        let diff = tracker.finish(&manifest_path);

        // 所有旧条目应该是 deleted
        assert!(diff.added.is_empty());
        assert_eq!(diff.deleted.len(), 1);
    }

    #[test]
    fn tracker_finish_handles_unknown_generator() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.json");

        // 创建现有 manifest，使用不同的 generator_id
        let mut manifest = Manifest::default();
        let entries = HashMap::from([
            ("Model1".to_string(), "model1.ts".to_string()),
        ]);
        manifest.update_generator("other_generator".to_string(), entries);
        manifest.save(&manifest_path).unwrap();

        // 使用不同 generator_id 的 tracker
        let mut tracker = ManifestTracker::new("models");
        tracker.track("Model2", "model2.ts");

        let diff = tracker.finish(&manifest_path);

        // 应该只有 added，没有 deleted（因为 generator_id 不同）
        assert_eq!(diff.added.len(), 1);
        assert!(diff.deleted.is_empty());
    }

    // ==================== update_manifest 函数测试 ====================

    #[test]
    fn update_manifest_creates_new_when_missing() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.json");

        let entries = HashMap::from([
            ("Model1".to_string(), "model1.ts".to_string()),
        ]);

        let result = update_manifest(
            &manifest_path,
            "models".to_string(),
            entries,
            "hash123",
            "3.0.0",
        );

        assert!(result.is_ok());
        assert!(manifest_path.exists());
    }

    #[test]
    fn update_manifest_updates_existing() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.json");

        // 创建初始 manifest
        let initial_entries = HashMap::from([
            ("Model1".to_string(), "model1.ts".to_string()),
        ]);
        update_manifest(
            &manifest_path,
            "models".to_string(),
            initial_entries,
            "hash1",
            "1.0.0",
        ).unwrap();

        // 更新 manifest
        let updated_entries = HashMap::from([
            ("Model2".to_string(), "model2.ts".to_string()),
        ]);
        let result = update_manifest(
            &manifest_path,
            "models".to_string(),
            updated_entries,
            "hash2",
            "2.0.0",
        );

        assert!(result.is_ok());
        let manifest = Manifest::load(&manifest_path).unwrap();
        assert_eq!(manifest.openapi_hash, "hash2");
        assert_eq!(manifest.openapi_version, "2.0.0");
        let entries = manifest.get_generator_entries("models").unwrap();
        assert!(entries.contains_key("Model2"));
        assert!(!entries.contains_key("Model1"));
    }

    #[test]
    fn update_manifest_updates_openapi_hash() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.json");

        let entries = HashMap::new();
        let result = update_manifest(
            &manifest_path,
            "models".to_string(),
            entries,
            "new_hash_value",
            "",
        );

        assert!(result.is_ok());
        let manifest = result.unwrap();
        assert_eq!(manifest.openapi_hash, "new_hash_value");
    }

    #[test]
    fn update_manifest_preserves_empty_hash() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.json");

        // 创建带有 hash 的 manifest
        let entries = HashMap::new();
        update_manifest(
            &manifest_path,
            "models".to_string(),
            entries,
            "original_hash",
            "1.0.0",
        ).unwrap();

        // 用空 hash 更新
        let entries = HashMap::new();
        let result = update_manifest(
            &manifest_path,
            "models".to_string(),
            entries,
            "", // 空 hash
            "",
        );

        assert!(result.is_ok());
        let manifest = result.unwrap();
        // 空 hash 不应该覆盖现有值
        assert_eq!(manifest.openapi_hash, "original_hash");
    }

    #[test]
    fn update_manifest_creates_parent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested").join("dir").join("manifest.json");

        let entries = HashMap::new();
        let result = update_manifest(
            &nested_path,
            "models".to_string(),
            entries,
            "",
            "",
        );

        assert!(result.is_ok());
        assert!(nested_path.exists());
    }
}
