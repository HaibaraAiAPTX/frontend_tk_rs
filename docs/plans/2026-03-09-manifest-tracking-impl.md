# Manifest 追踪系统实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为代码生成器添加 Manifest 追踪系统，实现文件变更检测、删除报告生成和强制桶文件更新。

**Architecture:** 在 swagger_gen crate 中新增 manifest 模块，提供 ManifestTracker 和 Reporter；修改现有生成命令集成追踪器；更新 TypeScript CLI 插件添加新参数。

**Tech Stack:** Rust (serde, chrono, sha2), TypeScript

---

## Task 1: 添加依赖项

**Files:**
- Modify: `crates/swagger_gen/Cargo.toml`

**Step 1: 添加所需依赖**

在 `crates/swagger_gen/Cargo.toml` 的 `[dependencies]` 部分添加：

```toml
chrono = { version = "0.4", features = ["serde"] }
sha2 = "0.10"
```

**Step 2: 验证依赖可用**

Run: `cd f:/Project/aptx-root/frontend_tk_rs && cargo check -p swagger_gen`
Expected: 编译成功，无错误

**Step 3: Commit**

```bash
git add crates/swagger_gen/Cargo.toml
git commit -m "chore: add chrono and sha2 dependencies for manifest tracking"
```

---

## Task 2: 创建 Manifest 数据结构

**Files:**
- Create: `crates/swagger_gen/src/manifest/mod.rs`
- Create: `crates/swagger_gen/src/manifest/schema.rs`
- Modify: `crates/swagger_gen/src/lib.rs`

**Step 1: 创建 manifest 模块入口**

创建 `crates/swagger_gen/src/manifest/mod.rs`：

```rust
mod schema;
mod tracker;
mod reporter;

pub use schema::*;
pub use tracker::*;
pub use reporter::*;
```

**Step 2: 创建数据结构定义**

创建 `crates/swagger_gen/src/manifest/schema.rs`：

```rust
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
```

**Step 3: 更新 lib.rs 导出模块**

修改 `crates/swagger_gen/src/lib.rs`：

```rust
pub mod core;
pub mod gen_api_trait;
pub mod manifest;
pub mod model_pipeline;
pub mod pipeline;
pub mod utils;
```

**Step 4: 验证编译**

Run: `cd f:/Project/aptx-root/frontend_tk_rs && cargo check -p swagger_gen`
Expected: 编译成功

**Step 5: Commit**

```bash
git add crates/swagger_gen/src/manifest/ crates/swagger_gen/src/lib.rs
git commit -m "feat(manifest): add manifest data structures"
```

---

## Task 3: 实现 ManifestTracker

**Files:**
- Create: `crates/swagger_gen/src/manifest/tracker.rs`

**Step 1: 创建追踪器实现**

创建 `crates/swagger_gen/src/manifest/tracker.rs`：

```rust
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
```

**Step 2: 更新 mod.rs 导出**

确保 `crates/swagger_gen/src/manifest/mod.rs` 包含正确的导出：

```rust
mod schema;
mod tracker;
mod reporter;

pub use schema::*;
pub use tracker::*;
pub use reporter::*;
```

**Step 3: 验证编译**

Run: `cd f:/Project/aptx-root/frontend_tk_rs && cargo check -p swagger_gen`
Expected: 编译成功

**Step 4: Commit**

```bash
git add crates/swagger_gen/src/manifest/
git commit -m "feat(manifest): implement ManifestTracker"
```

---

## Task 4: 实现删除报告生成器

**Files:**
- Create: `crates/swagger_gen/src/manifest/reporter.rs`

**Step 1: 创建报告生成器**

创建 `crates/swagger_gen/src/manifest/reporter.rs`：

```rust
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
        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write report: {}", e))
    }

    /// 生成 Markdown 报告
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str("# 代码生成删除报告\n\n");
        md.push_str(&format!("**生成时间**: {}\n", self.generated_at.format("%Y-%m-%d %H:%M:%S")));
        md.push_str(&format!("**生成器**: {}\n\n", self.generator_id));

        // 摘要
        md.push_str("## 摘要\n\n");
        md.push_str("| 操作 | 数量 |\n");
        md.push_str("|------|------|\n");
        md.push_str(&format!("| 删除 | {} |\n", self.summary.deleted_count));
        md.push_str(&format!("| 新增 | {} |\n", self.summary.added_count));
        md.push_str(&format!("| 未变更 | {} |\n\n", self.summary.unchanged_count));

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
```

**Step 2: 验证编译**

Run: `cd f:/Project/aptx-root/frontend_tk_rs && cargo check -p swagger_gen`
Expected: 编译成功

**Step 3: Commit**

```bash
git add crates/swagger_gen/src/manifest/reporter.rs
git commit -m "feat(manifest): implement deletion report generator"
```

---

## Task 5: 实现桶文件强制更新

**Files:**
- Modify: `crates/swagger_gen/src/pipeline/layout.rs`

**Step 1: 找到现有的桶文件生成逻辑**

先读取 `crates/swagger_gen/src/pipeline/layout.rs` 了解现有实现。

**Step 2: 添加强制更新函数**

在 `crates/swagger_gen/src/pipeline/layout.rs` 中添加或修改以下函数：

```rust
use std::path::Path;
use std::fs;

/// 强制更新指定目录的桶文件
pub fn force_update_barrel(relative_dir: &str, output_root: &Path) -> Result<(), String> {
    let dir_path = output_root.join(relative_dir);

    if !dir_path.exists() || !dir_path.is_dir() {
        return Ok(()); // 目录不存在，跳过
    }

    // 扫描目录下的所有 .ts 文件（排除 index.ts）
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

    // 生成 index.ts 内容
    let content: String = files
        .iter()
        .map(|f| format!("export * from './{}';\n", f))
        .collect();

    // 直接写入
    let index_path = dir_path.join("index.ts");
    fs::write(&index_path, content)
        .map_err(|e| format!("Failed to write barrel file {:?}: {}", index_path, e))?;

    Ok(())
}

/// 更新目录及其所有父级的桶文件（以 output_root 为边界）
pub fn update_barrel_with_parents(relative_dir: &str, output_root: &Path) -> Result<(), String> {
    // 更新当前目录
    force_update_barrel(relative_dir, output_root)?;

    // 更新所有父级目录（不超过 output_root）
    let mut current = Path::new(relative_dir);
    while let Some(parent) = current.parent() {
        if parent.as_os_str().is_empty() {
            break;
        }

        // 边界检查：不超过 output_root
        let parent_path = output_root.join(parent);
        if parent_path == output_root {
            break;
        }

        force_update_barrel(parent.to_str().unwrap(), output_root)?;
        current = parent;
    }

    Ok(())
}
```

**Step 3: 在 mod.rs 中导出新函数**

确保 `crates/swagger_gen/src/pipeline/mod.rs` 导出新函数：

```rust
pub use layout::*;
```

**Step 4: 验证编译**

Run: `cd f:/Project/aptx-root/frontend_tk_rs && cargo check -p swagger_gen`
Expected: 编译成功

**Step 5: Commit**

```bash
git add crates/swagger_gen/src/pipeline/layout.rs crates/swagger_gen/src/pipeline/mod.rs
git commit -m "feat(barrel): add force update barrel file functions"
```

---

## Task 6: 集成到 model:gen 命令

**Files:**
- Modify: `crates/node_binding/src/built_in/model_gen.rs`

**Step 1: 添加新参数**

修改 `crates/node_binding/src/built_in/model_gen.rs`：

```rust
use std::{fs, path::Path, collections::HashMap};

use aptx_frontend_tk_binding_plugin::utils::ensure_path;
use clap::Parser;
use swagger_gen::manifest::{
    ManifestTracker, update_manifest, generate_reports, ManifestDiff,
};
use swagger_gen::pipeline::update_barrel_with_parents;
use swagger_gen::model_pipeline::{generate_model_files, generate_model_files_with_existing, ModelRenderStyle};
use swagger_tk::model::OpenAPIObject;

use super::model_enum_plan::load_existing_enums_from_model_files;

#[derive(Debug, Clone, Parser)]
pub struct ModelGenOps {
  #[arg(long)]
  output: String,

  #[arg(long, default_value = "module")]
  style: String,

  #[arg(long)]
  name: Option<Vec<String>>,

  #[arg(long, default_value = "false")]
  preserve: bool,

  /// Disable manifest tracking
  #[arg(long, default_value = "false")]
  no_manifest: bool,

  /// Custom manifest directory (default: .generated)
  #[arg(long, default_value = ".generated")]
  manifest_dir: String,

  /// Preview mode: generate report without updating manifest
  #[arg(long, default_value = "false")]
  dry_run: bool,
}

pub fn run_model_gen(args: &[String], open_api: &OpenAPIObject) {
  let args: Vec<String> = std::iter::once("--".to_string())
    .chain(args.iter().cloned())
    .collect();
  let options = ModelGenOps::try_parse_from(args).unwrap();
  let output = Path::new(&options.output);
  ensure_path(output);
  let style = ModelRenderStyle::parse(&options.style).unwrap();
  let only_names = options.name.unwrap_or_default();

  // 创建追踪器
  let mut tracker = ManifestTracker::new("models");

  let models = if options.preserve {
    let existing_enums = load_existing_enums_from_model_files(output);
    match existing_enums {
      Some(enums) => {
        generate_model_files_with_existing(open_api, style, &only_names, &enums).unwrap()
      }
      None => {
        generate_model_files(open_api, style, &only_names).unwrap()
      }
    }
  } else {
    generate_model_files(open_api, style, &only_names).unwrap()
  };

  // 写入文件并追踪
  for (name, content) in &models {
    let file_name = format!("{}.ts", name);
    fs::write(output.join(&file_name)).unwrap();
    tracker.track(name, file_name);
  }

  // 处理 manifest
  if !options.no_manifest {
    let manifest_path = output.join(&options.manifest_dir).join("manifest.json");

    // 计算差异
    let diff = tracker.finish(&manifest_path);

    // 生成报告
    if let Err(e) = generate_reports(&diff, output, &options.manifest_dir) {
      eprintln!("Warning: Failed to generate reports: {}", e);
    }

    // 更新 manifest（非 dry_run 模式）
    if !options.dry_run {
      if let Err(e) = update_manifest(
        &manifest_path,
        "models".to_string(),
        tracker.entries().clone(),
        "", // openapi_hash
        "", // openapi_version
      ) {
        eprintln!("Warning: Failed to update manifest: {}", e);
      }
    }

    // 输出摘要
    if diff.has_changes() {
      println!("Manifest changes:");
      println!("  Added: {} files", diff.added.len());
      println!("  Deleted: {} files", diff.deleted.len());
      println!("  Unchanged: {} files", diff.unchanged.len());
    }

    // 更新桶文件
    if let Err(e) = update_barrel_with_parents("models", output) {
      eprintln!("Warning: Failed to update barrel files: {}", e);
    }
  }
}
```

**Step 2: 验证编译**

Run: `cd f:/Project/aptx-root/frontend_tk_rs && cargo check -p node_binding`
Expected: 编译成功

**Step 3: Commit**

```bash
git add crates/node_binding/src/built_in/model_gen.rs
git commit -m "feat(model): integrate manifest tracking into model:gen command"
```

---

## Task 7: 集成到 terminal:codegen 命令

**Files:**
- Modify: `crates/node_binding/src/built_in/terminal_codegen.rs`

**Step 1: 读取现有实现**

先读取 `crates/node_binding/src/built_in/terminal_codegen.rs` 了解现有结构。

**Step 2: 添加 manifest 追踪参数和逻辑**

在 TerminalCodegenOps 结构体中添加新参数，并在生成完成后集成 manifest 追踪逻辑。

关键修改点：
1. 添加 `--no-manifest`、`--manifest-dir`、`--dry-run` 参数
2. 根据不同的 terminal 类型（functions/react-query/vue-query）使用对应的 generator_id
3. 生成完成后调用 manifest 追踪逻辑
4. 更新桶文件

**Step 3: 验证编译**

Run: `cd f:/Project/aptx-root/frontend_tk_rs && cargo check -p node_binding`
Expected: 编译成功

**Step 4: Commit**

```bash
git add crates/node_binding/src/built_in/terminal_codegen.rs
git commit -m "feat(terminal): integrate manifest tracking into terminal:codegen command"
```

---

## Task 8: 更新 TypeScript CLI 插件 - model

**Files:**
- Modify: `packages/frontend-tk-plugin-model/src/index.ts`

**Step 1: 添加新参数和示例**

修改 `packages/frontend-tk-plugin-model/src/index.ts` 中的 `model:gen` 命令：

```typescript
{
  name: 'model:gen',
  summary: 'Generate TypeScript model declarations from OpenAPI schemas',
  description: 'Generates TypeScript type definitions from OpenAPI schema definitions.',
  options: [
    ...commonModelOptions,
    {
      flags: '--style <declaration|module>',
      description: 'Model output style',
      defaultValue: 'module',
    },
    {
      flags: '--name <schema>',
      description: 'Generate specific schema names only; repeatable',
      required: false,
    },
    {
      flags: '--preserve',
      description: 'Preserve existing translated enum names when regenerating models',
      defaultValue: false,
    },
    // 新增参数
    {
      flags: '--no-manifest',
      description: 'Disable manifest tracking and deletion report generation',
      defaultValue: false,
    },
    {
      flags: '--manifest-dir <path>',
      description: 'Custom directory for manifest files (default: .generated)',
      defaultValue: '.generated',
    },
    {
      flags: '--dry-run',
      description: 'Preview mode: generate deletion report without updating manifest',
      defaultValue: false,
    },
  ],
  examples: [
    'aptx-ft model gen --input openapi.json --output ./src/models',
    'aptx-ft model gen --input openapi.json --output ./src/models --style module',
    'aptx-ft model gen --input openapi.json --output ./src/models --dry-run',
    'aptx-ft model gen --input openapi.json --output ./src/models --no-manifest',
  ],
  handler: async (ctx: PluginContext, args: Record<string, unknown>) => {
    const binding = ctx.binding as any;
    if (typeof binding.runCli === 'function') {
      const options: string[] = [];
      if (args.output) options.push('--output', String(args.output));
      if (args.style) options.push('--style', String(args.style));
      if (args.name) {
        const names = Array.isArray(args.name) ? args.name : [args.name];
        names.forEach((n: string) => options.push('--name', n));
      }
      if (args.preserve) options.push('--preserve');
      // 新增参数
      if (args.noManifest) options.push('--no-manifest');
      if (args.manifestDir) options.push('--manifest-dir', String(args.manifestDir));
      if (args.dryRun) options.push('--dry-run');

      binding.runCli({
        input: args.input as string | undefined,
        command: 'model:gen',
        options,
        plugin: args.plugin as string[] | undefined,
      });
    } else {
      ctx.log('Native binding runCli not available for command: model:gen');
    }
  },
},
```

**Step 2: 验证 TypeScript 编译**

Run: `cd f:/Project/aptx-root/frontend_tk_rs && pnpm -F @aptx/frontend-tk-plugin-model build`
Expected: 编译成功

**Step 3: Commit**

```bash
git add packages/frontend-tk-plugin-model/src/index.ts
git commit -m "feat(cli): add manifest tracking options to model:gen command"
```

---

## Task 9: 更新 TypeScript CLI 插件 - aptx

**Files:**
- Modify: `packages/frontend-tk-plugin-aptx/src/functions.ts`
- Modify: `packages/frontend-tk-plugin-aptx/src/react-query.ts`
- Modify: `packages/frontend-tk-plugin-aptx/src/vue-query.ts`

**Step 1: 更新 functions.ts**

在 `packages/frontend-tk-plugin-aptx/src/functions.ts` 中添加新参数：

```typescript
options: [
  // ... 现有参数 ...
  {
    flags: '--no-manifest',
    description: 'Disable manifest tracking and deletion report generation',
    defaultValue: false,
  },
  {
    flags: '--manifest-dir <path>',
    description: 'Custom directory for manifest files (default: .generated)',
    defaultValue: '.generated',
  },
  {
    flags: '--dry-run',
    description: 'Preview mode: generate deletion report without updating manifest',
    defaultValue: false,
  },
],
examples: [
  'aptx-ft aptx functions -i openapi.json -o ./generated',
  'aptx-ft aptx functions -i openapi.json -o ./generated --dry-run',
  'aptx-ft aptx functions -i openapi.json -o ./generated --no-manifest',
],
```

并在 handler 中传递这些参数：

```typescript
// 在 handler 中
if (args.noManifest) options.push('--no-manifest');
if (args.manifestDir) options.push('--manifest-dir', String(args.manifestDir));
if (args.dryRun) options.push('--dry-run');
```

**Step 2: 更新 react-query.ts 和 vue-query.ts**

同样的修改应用到这两个文件。

**Step 3: 验证 TypeScript 编译**

Run: `cd f:/Project/aptx-root/frontend_tk_rs && pnpm -F @aptx/frontend-tk-plugin-aptx build`
Expected: 编译成功

**Step 4: Commit**

```bash
git add packages/frontend-tk-plugin-aptx/src/
git commit -m "feat(cli): add manifest tracking options to aptx commands"
```

---

## Task 10: 完整构建和测试

**Step 1: 完整构建**

Run: `cd f:/Project/aptx-root/frontend_tk_rs && cargo build --release`
Expected: 构建成功

**Step 2: 构建 TypeScript 包**

Run: `cd f:/Project/aptx-root/frontend_tk_rs && pnpm build`
Expected: 构建成功

**Step 3: 手动测试**

使用测试 OpenAPI 文件运行命令：

```bash
# 测试模型生成
aptx-ft model:gen --input test-openapi.json --output ./test-output

# 验证 .generated 目录
ls ./test-output/.generated/

# 测试 dry-run 模式
aptx-ft model:gen --input test-openapi.json --output ./test-output --dry-run

# 测试 aptx:functions
aptx-ft aptx:functions --input test-openapi.json --output ./test-output
```

**Step 4: 最终 Commit**

```bash
git add .
git commit -m "chore: final build and test verification"
```

---

## 执行顺序总结

1. Task 1: 添加依赖项
2. Task 2: 创建 Manifest 数据结构
3. Task 3: 实现 ManifestTracker
4. Task 4: 实现删除报告生成器
5. Task 5: 实现桶文件强制更新
6. Task 6: 集成到 model:gen 命令
7. Task 7: 集成到 terminal:codegen 命令
8. Task 8: 更新 TypeScript CLI 插件 - model
9. Task 9: 更新 TypeScript CLI 插件 - aptx
10. Task 10: 完整构建和测试
