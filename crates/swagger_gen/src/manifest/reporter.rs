use super::ManifestDiff;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 删除报告（JSON 格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionReport {
    /// 报告版本
    pub version: String,
    /// 生成时间
    pub generated_at: DateTime<Utc>,
    /// 生成器 ID
    pub generator_id: String,
    /// 被删除的条目
    pub deleted: Vec<ReportEntry>,
    /// 新增的条目名称
    pub added: Vec<String>,
    /// 摘要
    pub summary: ReportSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportEntry {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub deleted_count: usize,
    pub added_count: usize,
    pub unchanged_count: usize,
}

impl DeletionReport {
    /// 从 ManifestDiff 创建报告
    pub fn from_diff(diff: &ManifestDiff) -> Self {
        let deleted: Vec<ReportEntry> = diff
            .deleted
            .iter()
            .map(|(name, path)| ReportEntry {
                name: name.clone(),
                path: path.clone(),
            })
            .collect();

        let added: Vec<String> = diff.added.iter().map(|(name, _)| name.clone()).collect();

        Self {
            version: "1.0".to_string(),
            generated_at: Utc::now(),
            generator_id: diff.generator_id.clone(),
            deleted,
            added,
            summary: ReportSummary {
                deleted_count: diff.deleted.len(),
                added_count: diff.added.len(),
                unchanged_count: diff.unchanged.len(),
            },
        }
    }

    /// 保存 JSON 报告
    pub fn save_json(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create report directory: {}", e))?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize report: {}", e))?;
        std::fs::write(path, content).map_err(|e| format!("Failed to write report: {}", e))
    }

    /// 生成 Markdown 报告
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str("# 代码生成删除报告\n\n");
        md.push_str(&format!(
            "**生成时间**: {}\n",
            self.generated_at.format("%Y-%m-%d %H:%M:%S")
        ));
        md.push_str(&format!("**生成器**: {}\n\n", self.generator_id));

        // 摘要
        md.push_str("## 摘要\n\n");
        md.push_str("| 操作 | 数量 |\n");
        md.push_str("|------|------|\n");
        md.push_str(&format!("| 删除 | {} |\n", self.summary.deleted_count));
        md.push_str(&format!("| 新增 | {} |\n", self.summary.added_count));
        md.push_str(&format!(
            "| 未变更 | {} |\n\n",
            self.summary.unchanged_count
        ));

        // 删除的文件
        if !self.deleted.is_empty() {
            md.push_str("## 删除的文件\n\n");
            md.push_str("| 名称 | 文件路径 |\n");
            md.push_str("|------|----------|\n");
            for entry in &self.deleted {
                md.push_str(&format!("| `{}` | `{}` |\n", entry.name, entry.path));
            }
            md.push_str("\n");
        }

        // 新增的文件
        if !self.added.is_empty() {
            md.push_str("## 新增的文件\n\n");
            for name in &self.added {
                md.push_str(&format!("- `{}`\n", name));
            }
            md.push_str("\n");
        }

        // LLM 建议
        if !self.deleted.is_empty() {
            md.push_str("---\n\n");
            md.push_str("## LLM 后续操作建议\n\n");
            md.push_str("以下文件已被标记为待删除，请检查项目中的引用：\n\n");
            md.push_str("1. **搜索引用**: 在项目中搜索以下名称：\n");
            for entry in &self.deleted {
                md.push_str(&format!("   - `{}`\n", entry.name));
            }
            md.push_str("\n2. **更新代码**: 将引用替换为新的 API 或模型\n");
            md.push_str("\n3. **手动删除**: 确认无引用后，删除以下文件：\n");
            for entry in &self.deleted {
                md.push_str(&format!("   - `{}`\n", entry.path));
            }
        }

        md
    }

    /// 保存 Markdown 报告
    pub fn save_markdown(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create report directory: {}", e))?;
        }
        std::fs::write(path, self.to_markdown())
            .map_err(|e| format!("Failed to write markdown report: {}", e))
    }
}

/// 生成并保存报告
pub fn generate_reports(
    diff: &ManifestDiff,
    output_dir: &Path,
    manifest_dir: &str,
) -> Result<DeletionReport, String> {
    let report = DeletionReport::from_diff(diff);

    let generated_dir = output_dir.join(manifest_dir);

    report.save_json(&generated_dir.join("deletion-report.json"))?;
    report.save_markdown(&generated_dir.join("deletion-report.md"))?;

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::ManifestDiff;
    use tempfile::TempDir;

    // ==================== DeletionReport 测试 ====================

    #[test]
    fn deletion_report_from_diff_converts_added() {
        let diff = ManifestDiff {
            generator_id: "models".to_string(),
            added: vec![
                ("Model1".to_string(), "model1.ts".to_string()),
                ("Model2".to_string(), "model2.ts".to_string()),
            ],
            deleted: vec![],
            unchanged: vec![],
        };

        let report = DeletionReport::from_diff(&diff);

        assert_eq!(report.added.len(), 2);
        assert!(report.added.contains(&"Model1".to_string()));
        assert!(report.added.contains(&"Model2".to_string()));
    }

    #[test]
    fn deletion_report_from_diff_converts_deleted() {
        let diff = ManifestDiff {
            generator_id: "models".to_string(),
            added: vec![],
            deleted: vec![("OldModel".to_string(), "old_model.ts".to_string())],
            unchanged: vec![],
        };

        let report = DeletionReport::from_diff(&diff);

        assert_eq!(report.deleted.len(), 1);
        assert_eq!(report.deleted[0].name, "OldModel");
        assert_eq!(report.deleted[0].path, "old_model.ts");
    }

    #[test]
    fn deletion_report_from_diff_calculates_summary() {
        let diff = ManifestDiff {
            generator_id: "models".to_string(),
            added: vec![("Model1".to_string(), "model1.ts".to_string())],
            deleted: vec![("OldModel".to_string(), "old_model.ts".to_string())],
            unchanged: vec!["UnchangedModel".to_string()],
        };

        let report = DeletionReport::from_diff(&diff);

        assert_eq!(report.summary.added_count, 1);
        assert_eq!(report.summary.deleted_count, 1);
        assert_eq!(report.summary.unchanged_count, 1);
    }

    #[test]
    fn deletion_report_from_diff_handles_empty_diff() {
        let diff = ManifestDiff {
            generator_id: "models".to_string(),
            added: vec![],
            deleted: vec![],
            unchanged: vec![],
        };

        let report = DeletionReport::from_diff(&diff);

        assert!(report.added.is_empty());
        assert!(report.deleted.is_empty());
        assert_eq!(report.summary.added_count, 0);
        assert_eq!(report.summary.deleted_count, 0);
        assert_eq!(report.summary.unchanged_count, 0);
    }

    // ==================== JSON 保存测试 ====================

    #[test]
    fn deletion_report_save_json_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("report.json");

        let diff = ManifestDiff {
            generator_id: "models".to_string(),
            added: vec![],
            deleted: vec![],
            unchanged: vec![],
        };
        let report = DeletionReport::from_diff(&diff);

        let result = report.save_json(&json_path);

        assert!(result.is_ok());
        assert!(json_path.exists());
    }

    #[test]
    fn deletion_report_save_json_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("nested")
            .join("dir")
            .join("report.json");

        let diff = ManifestDiff {
            generator_id: "models".to_string(),
            added: vec![],
            deleted: vec![],
            unchanged: vec![],
        };
        let report = DeletionReport::from_diff(&diff);

        let result = report.save_json(&nested_path);

        assert!(result.is_ok());
        assert!(nested_path.exists());
    }

    #[test]
    fn deletion_report_save_json_is_valid_json() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("report.json");

        let diff = ManifestDiff {
            generator_id: "models".to_string(),
            added: vec![("Model1".to_string(), "model1.ts".to_string())],
            deleted: vec![("OldModel".to_string(), "old_model.ts".to_string())],
            unchanged: vec![],
        };
        let report = DeletionReport::from_diff(&diff);
        report.save_json(&json_path).unwrap();

        let content = std::fs::read_to_string(&json_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed["generator_id"], "models");
        assert_eq!(parsed["summary"]["added_count"], 1);
        assert_eq!(parsed["summary"]["deleted_count"], 1);
    }

    // ==================== Markdown 测试 ====================

    #[test]
    fn deletion_report_to_markdown_includes_header() {
        let diff = ManifestDiff {
            generator_id: "models".to_string(),
            added: vec![],
            deleted: vec![],
            unchanged: vec![],
        };
        let report = DeletionReport::from_diff(&diff);

        let md = report.to_markdown();

        assert!(md.contains("# 代码生成删除报告"));
        assert!(md.contains("**生成时间**:"));
        assert!(md.contains("**生成器**: models"));
    }

    #[test]
    fn deletion_report_to_markdown_includes_summary_table() {
        let diff = ManifestDiff {
            generator_id: "models".to_string(),
            added: vec![("Model1".to_string(), "model1.ts".to_string())],
            deleted: vec![("OldModel".to_string(), "old_model.ts".to_string())],
            unchanged: vec!["UnchangedModel".to_string()],
        };
        let report = DeletionReport::from_diff(&diff);

        let md = report.to_markdown();

        assert!(md.contains("## 摘要"));
        assert!(md.contains("| 删除 | 1 |"));
        assert!(md.contains("| 新增 | 1 |"));
        assert!(md.contains("| 未变更 | 1 |"));
    }

    #[test]
    fn deletion_report_to_markdown_includes_deleted_files() {
        let diff = ManifestDiff {
            generator_id: "models".to_string(),
            added: vec![],
            deleted: vec![("OldModel".to_string(), "old_model.ts".to_string())],
            unchanged: vec![],
        };
        let report = DeletionReport::from_diff(&diff);

        let md = report.to_markdown();

        assert!(md.contains("## 删除的文件"));
        assert!(md.contains("`OldModel`"));
        assert!(md.contains("`old_model.ts`"));
    }

    #[test]
    fn deletion_report_to_markdown_includes_added_files() {
        let diff = ManifestDiff {
            generator_id: "models".to_string(),
            added: vec![("NewModel".to_string(), "new_model.ts".to_string())],
            deleted: vec![],
            unchanged: vec![],
        };
        let report = DeletionReport::from_diff(&diff);

        let md = report.to_markdown();

        assert!(md.contains("## 新增的文件"));
        assert!(md.contains("`NewModel`"));
    }

    #[test]
    fn deletion_report_to_markdown_includes_llm_suggestions() {
        let diff = ManifestDiff {
            generator_id: "models".to_string(),
            added: vec![],
            deleted: vec![("OldModel".to_string(), "old_model.ts".to_string())],
            unchanged: vec![],
        };
        let report = DeletionReport::from_diff(&diff);

        let md = report.to_markdown();

        assert!(md.contains("## LLM 后续操作建议"));
        assert!(md.contains("搜索引用"));
        assert!(md.contains("更新代码"));
        assert!(md.contains("手动删除"));
    }

    #[test]
    fn deletion_report_to_markdown_handles_empty_deleted() {
        let diff = ManifestDiff {
            generator_id: "models".to_string(),
            added: vec![("Model1".to_string(), "model1.ts".to_string())],
            deleted: vec![],
            unchanged: vec![],
        };
        let report = DeletionReport::from_diff(&diff);

        let md = report.to_markdown();

        assert!(!md.contains("## LLM 后续操作建议"));
        assert!(!md.contains("## 删除的文件"));
    }

    #[test]
    fn deletion_report_save_markdown_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let md_path = temp_dir.path().join("report.md");

        let diff = ManifestDiff {
            generator_id: "models".to_string(),
            added: vec![],
            deleted: vec![],
            unchanged: vec![],
        };
        let report = DeletionReport::from_diff(&diff);

        let result = report.save_markdown(&md_path);

        assert!(result.is_ok());
        assert!(md_path.exists());
    }

    // ==================== generate_reports 函数测试 ====================

    #[test]
    fn generate_reports_creates_both_files() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path();

        let diff = ManifestDiff {
            generator_id: "models".to_string(),
            added: vec![],
            deleted: vec![],
            unchanged: vec![],
        };

        let result = generate_reports(&diff, output_dir, ".generated");

        assert!(result.is_ok());
        assert!(output_dir.join(".generated/deletion-report.json").exists());
        assert!(output_dir.join(".generated/deletion-report.md").exists());
    }

    #[test]
    fn generate_reports_returns_correct_report() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path();

        let diff = ManifestDiff {
            generator_id: "models".to_string(),
            added: vec![("Model1".to_string(), "model1.ts".to_string())],
            deleted: vec![("OldModel".to_string(), "old_model.ts".to_string())],
            unchanged: vec![],
        };

        let result = generate_reports(&diff, output_dir, ".generated");

        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.generator_id, "models");
        assert_eq!(report.summary.added_count, 1);
        assert_eq!(report.summary.deleted_count, 1);
    }
}
