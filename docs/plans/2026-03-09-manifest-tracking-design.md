# 代码生成器 Manifest 追踪系统设计

**日期**: 2026-03-09
**状态**: 已批准
**作者**: Claude

## 概述

为代码生成器添加 Manifest 追踪系统，用于：
1. 追踪已生成的文件
2. 检测并报告被删除的模型和 API
3. 强制更新桶文件（index.ts）
4. 为 LLM 提供变更报告，便于后续项目调整

## 目标

- 当 OpenAPI 文档变动时，能够识别并报告需要删除的老旧代码
- 提供结构化的变更报告供 LLM 使用
- 简化新插件的接入流程
- 保持现有命令的兼容性

## 文件结构

```
<output-dir>/
├── .generated/
│   ├── manifest.json           # 当前生成清单
│   ├── deletion-report.json    # 删除报告（JSON）
│   └── deletion-report.md      # 删除报告（Markdown）
├── models/
│   ├── index.ts
│   └── *.ts
└── api/
    ├── index.ts
    ├── functions/
    │   ├── index.ts
    │   └── *.ts
    ├── react-query/
    │   └── ...
    └── vue-query/
        └── ...
```

## 数据结构

### manifest.json

```json
{
  "version": "1.0",
  "generatedAt": "2026-03-09T10:30:00Z",
  "openapiHash": "a1b2c3d4e5f6...",
  "openapiVersion": "3.0.0",
  "generatorVersion": "0.5.0",
  "generators": {
    "models": {
      "User": "models/User.ts",
      "CreateUserRequest": "models/CreateUserRequest.ts"
    },
    "functions": {
      "getUser": "api/functions/getUser.ts"
    },
    "react-query": {
      "getUser": "api/react-query/useGetUser.ts"
    },
    "vue-query": {
      "getUser": "api/vue-query/useGetUser.ts"
    }
  }
}
```

### deletion-report.json

```json
{
  "version": "1.0",
  "generatedAt": "2026-03-09T10:30:00Z",
  "generatorId": "models",
  "deleted": [
    {
      "name": "OldUser",
      "path": "models/OldUser.ts"
    }
  ],
  "added": ["NewUser"],
  "summary": {
    "deletedCount": 1,
    "addedCount": 1,
    "unchangedCount": 20
  }
}
```

### deletion-report.md

```markdown
# 代码生成删除报告

**生成时间**: 2026-03-09 10:30:00
**生成器**: models
**OpenAPI Hash**: `a1b2c3d4e5f6...`

## 摘要

| 操作 | 数量 |
|------|------|
| 删除 | 1 |
| 新增 | 1 |
| 未变更 | 20 |

## 删除的文件

| 名称 | 文件路径 |
|------|----------|
| `OldUser` | `models/OldUser.ts` |

## 新增的文件

- `NewUser`

---

## LLM 后续操作建议

以下文件已被标记为待删除，请检查项目中的引用：

1. **搜索引用**: 在项目中搜索 `OldUser`
2. **更新代码**: 将引用替换为新的 API 或模型
3. **手动删除**: 确认无引用后，删除 `models/OldUser.ts`
```

## Rust 代码结构

```
crates/swagger_gen/src/
├── manifest/
│   ├── mod.rs              # 模块入口
│   ├── schema.rs           # Manifest 数据结构
│   ├── tracker.rs          # 追踪器 + GeneratorPlugin trait
│   └── reporter.rs         # 删除报告生成器
├── barrel/
│   └── generator.rs        # 桶文件生成器（修改为强制更新）
└── pipeline/
    └── ...                 # 现有生成逻辑集成 tracker
```

### 核心 Trait 和结构

```rust
// manifest/tracker.rs

/// 生成器插件需要实现此 trait
pub trait GeneratorPlugin {
    /// 生成器唯一标识（用于 manifest 中的 key）
    fn generator_id(&self) -> &str;

    /// 获取输出目录（用于桶文件生成）
    fn output_dir(&self) -> &str;
}

/// 统一的文件追踪器
pub struct ManifestTracker {
    generator_id: String,
    entries: HashMap<String, String>,  // name -> path
}

impl ManifestTracker {
    /// 创建新的追踪器（每个生成器调用）
    pub fn new(generator_id: impl Into<String>) -> Self { ... }

    /// 记录生成的文件
    pub fn track(&mut self, name: impl Into<String>, path: impl Into<String>) { ... }

    /// 完成追踪，返回变更结果
    pub fn finish(self, manifest_path: &Path) -> ManifestDiff { ... }
}

/// 变更对比结果
pub struct ManifestDiff {
    pub generator_id: String,
    pub added: Vec<(String, String)>,      // (name, path)
    pub deleted: Vec<(String, String)>,    // (name, path)
    pub unchanged: Vec<String>,            // names
}
```

### 桶文件生成器

```rust
// barrel/generator.rs

pub struct BarrelGenerator;

impl BarrelGenerator {
    /// 强制更新指定目录的桶文件
    pub fn force_update(relative_dir: &str, output_root: &Path) -> Result<()> { ... }

    /// 更新目录及其所有父级的桶文件（以 output_root 为边界）
    pub fn update_with_parents(relative_dir: &str, output_root: &Path) -> Result<()> {
        Self::force_update(relative_dir, output_root)?;

        let mut current = Path::new(relative_dir);
        while let Some(parent) = current.parent() {
            if parent.as_os_str().is_empty() {
                break;
            }

            // 边界检查：不超过 output_root
            let parent_path = output_root.join(parent);
            if !parent_path.starts_with(output_root) || parent_path == output_root {
                break;
            }

            Self::force_update(parent.to_str().unwrap(), output_root)?;
            current = parent;
        }

        Ok(())
    }
}
```

## 生成流程

每个生成命令执行时：

```
1. 读取 .generated/manifest.json（如存在）
2. 创建 ManifestTracker::new("generator-id")
3. 执行生成逻辑，调用 tracker.track(name, path)
4. 调用 tracker.finish() → 返回 ManifestDiff
5. 生成 deletion-report.json / .md
6. 更新 manifest.json
7. 强制更新桶文件（含父级，以 output_root 为边界）
```

## 新插件接入

新插件只需 3 步：

```rust
// 1. 实现 GeneratorPlugin trait
impl GeneratorPlugin for MyGenerator {
    fn generator_id(&self) -> &str { "my-generator" }
    fn output_dir(&self) -> &str { "api/my-generator" }
}

// 2. 在生成逻辑中使用 Tracker
let mut tracker = ManifestTracker::new(self.generator_id());
// ... 生成文件 ...
tracker.track("MyType", "path/to/MyType.ts");

// 3. 完成追踪
let diff = tracker.finish(&manifest_path);

// 4. 生成报告 + 更新桶文件
ReportGenerator::generate(&diff, output);
BarrelGenerator::update_with_parents(self.output_dir(), output);
```

## CLI 接口

### 现有命令变更

所有生成命令执行后会自动：
1. 更新 `.generated/manifest.json`
2. 生成 `.generated/deletion-report.json` 和 `.generated/deletion-report.md`
3. 强制更新相关目录的 `index.ts` 桶文件

### 新增参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `--no-manifest` | boolean | false | 禁用 manifest 追踪，使用旧逻辑 |
| `--manifest-dir` | string | `.generated` | manifest 文件存放目录 |
| `--dry-run` | boolean | false | 预览模式，只生成报告不更新 manifest |

### 示例

```bash
# 默认行为：生成代码 + manifest 追踪
frontend-tk model:gen --input openapi.json --output ./src/generated

# 禁用 manifest 追踪（旧逻辑）
frontend-tk model:gen --input openapi.json --output ./src/generated --no-manifest

# 预览模式
frontend-tk model:gen --input openapi.json --output ./src/generated --dry-run

# 自定义 manifest 目录
frontend-tk model:gen --input openapi.json --output ./src/generated --manifest-dir ./meta
```

## 决策记录

| 决策点 | 选择 | 理由 |
|--------|------|------|
| 判断标准 | 基于版本记录（manifest） | 可控、可追溯 |
| 清单内容 | 分层记录（基础清单） | 简洁，不冗余 |
| 清单组织 | 集中式 | 便于全局视图和报告生成 |
| 输出格式 | JSON + Markdown | JSON 供程序处理，Markdown 供 LLM 阅读 |
| 删除策略 | 只生成报告，不实际删除 | 更安全，用户手动确认 |
| 桶文件策略 | 始终强制更新 | 符合代码生成场景 |
| 架构 | 集成到 Rust 核心生成流程 | 性能好，一致性高 |
| 更新策略 | 渐进式更新 | 符合独立命令架构 |
| 桶文件边界 | 以 output_root 为边界 | 简单直接，不越界 |

## 待办事项

- [ ] 实现 `manifest/schema.rs` - 数据结构定义
- [ ] 实现 `manifest/tracker.rs` - 追踪器和 GeneratorPlugin trait
- [ ] 实现 `manifest/reporter.rs` - 报告生成器
- [ ] 修改 `barrel/generator.rs` - 强制更新逻辑
- [ ] 集成到现有生成命令
- [ ] 更新 TypeScript CLI 插件参数定义
