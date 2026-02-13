# 代码生成器架构迁移文档

## 1. 概述

### 1.1 迁移目标

核心纯净化：将代码生成器核心与 `@aptx` 包引用解耦，实现插件化架构，使核心代码生成器不依赖任何特定的基础库包。

### 1.2 当前问题

在 `renderer.rs` 中存在以下硬编码的 `@aptx` 引用：

- 第 360 行：`import { createQueryDefinition } from "@aptx/api-query-adapter"`
- 第 390 行：`import { createMutationDefinition } from "@aptx/api-query-adapter"`
- 第 390-391 行：`import { hook_factory } from "@aptx/{terminal_package}"`
- 第 391 行：`import { getApiClient } from "@aptx/api-client"`
- 第 459 行：`import type { PerCallOptions } from "@aptx/api-client"`
- 第 466 行：`import { PerCallOptions } from "@aptx/api-client"`
- 第 486 行：默认回退路径仍使用 `@aptx/api-client`

这些硬编码违反了核心纯净化原则，使得代码生成器无法在不依赖 `@aptx` 包的情况下运行。

### 1.3 解决方案

引入插件系统，将 `@aptx` 相关功能抽象为插件，使核心代码生成器不直接依赖任何特定包。

---

## 2. 当前架构问题

### 2.1 硬编码的 @aptx 引用

**位置**：`crates/swagger_gen/src/pipeline/renderer.rs`

```rust
// 第 334-372 行：render_query_file 函数
format!(
    "import {{ createQueryDefinition }} from \"@aptx/api-query-adapter\";\n\
     import {{ {hook_factory} }} from \"@aptx/{terminal_package}\";\n\
     {client_import_lines}\
     import {{ {builder} }} from \"../../spec/endpoints/{namespace}/{operation_name}\";\n\
     {type_imports}\n\n\
     const normalizeInput = (input: {input_type}) => JSON.stringify(input ?? null);\n\n\
     export const {query_def} = createQueryDefinition<{input_type}, {output_type}>({{...\
```

**问题分类**：

| 类型 | 位置 | 说明 |
|------|------|------|
| 导入语句 | L360, L389-391 | 硬编码 `@aptx/api-query-adapter` 和 `@aptx/{terminal}` |
| 类型导入 | L459, L466 | 硬编码 `@aptx/api-client` 类型导入 |
| 默认回退 | L486 | 全局模式默认使用 `@aptx/api-client` |

### 2.2 耦合度分析

```
renderer.rs (核心)
    |
    +-- @aptx/api-query-adapter (紧耦合)
    +-- @aptx/react-query (紧耦合)
    +-- @aptx/vue-query (紧耦合)
    +-- @aptx/api-client (紧耦合)
```

### 2.3 影响

1. **灵活性受限**：无法切换到其他 API 客户端或 Query 库
2. **测试困难**：核心功能测试必须依赖 `@aptx` 包
3. **扩展性差**：添加新的终端类型需要修改核心代码
4. **版本耦合**：核心代码与 `@aptx` 包版本绑定

---

## 3. 目标架构

### 3.1 架构图

```
                    ┌─────────────────────────────────────┐
                    │      代码生成器核心 (Core)         │
                    │   - Parser                        │
                    │   - Transformer                   │
                    │   - Renderer (纯净化后)            │
                    │   - Writer                        │
                    └─────────────┬───────────────────────┘
                                  │
                    ┌─────────────▼───────────────────────┐
                    │      Plugin Registry               │
                    │   - register(plugin)              │
                    │   - get(id)                      │
                    │   - execute(id, context)          │
                    └─────────────┬───────────────────────┘
                                  │
        ┌─────────────────────────┼─────────────────────────┐
        │                       │                         │
┌───────▼────────┐    ┌────────▼────────┐    ┌────────▼────────┐
│ @aptx Plugin   │    │  Standard Plugin │    │ Custom Plugin   │
│                │    │                 │    │                 │
│ - Functions    │    │ - AxiosTs       │    │ - User-defined  │
│ - ReactQuery   │    │ - AxiosJs       │    │                 │
│ - VueQuery    │    │ - UniApp        │    │                 │
└────────────────┘    └─────────────────┘    └─────────────────┘
```

### 3.2 插件系统设计

#### 3.2.1 核心接口

```rust
/// 终端插件核心 Trait
pub trait TerminalPlugin: Send + Sync {
    /// 插件唯一标识符
    fn id(&self) -> &'static str;

    /// 插件名称
    fn name(&self) -> &'static str;

    /// 插件描述
    fn description(&self) -> &'static str;

    /// 渲染单个端点
    fn render_endpoint(
        &self,
        endpoint: &EndpointItem,
        ctx: &RenderContext
    ) -> Result<Vec<PlannedFile>, String>;

    /// 渲染所有端点（可选，用于批量优化）
    fn render_all(
        &self,
        input: &GeneratorInput,
        ctx: &RenderContext
    ) -> Result<Vec<PlannedFile>, String> {
        // 默认实现：逐个调用 render_endpoint
        let mut files = Vec::new();
        for endpoint in &input.endpoints {
            files.extend(self.render_endpoint(endpoint, ctx)?);
        }
        Ok(files)
    }
}

/// 渲染上下文
pub struct RenderContext {
    /// 模型导入配置
    pub model_import: Option<ModelImportConfig>,

    /// 客户端导入配置
    pub client_import: Option<ClientImportConfig>,

    /// 项目上下文
    pub project: ProjectContext,

    /// 用户自定义配置
    pub user_config: serde_json::Value,
}
```

#### 3.2.2 插件注册表

```rust
/// 插件注册表
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn TerminalPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// 注册插件
    pub fn register(&mut self, plugin: Box<dyn TerminalPlugin>) {
        let id = plugin.id().to_string();
        self.plugins.insert(id, plugin);
    }

    /// 获取插件
    pub fn get(&self, id: &str) -> Option<&dyn TerminalPlugin> {
        self.plugins.get(id).map(|p| p.as_ref())
    }

    /// 列出所有插件
    pub fn list(&self) -> Vec<&dyn TerminalPlugin> {
        self.plugins.values().map(|p| p.as_ref()).collect()
    }
}
```

---

## 4. 插件接口设计

### 4.1 TerminalPlugin Trait

```rust
use async_trait::async_trait;

/// 终端插件核心 Trait
#[async_trait]
pub trait TerminalPlugin: Send + Sync {
    // === 基础信息 ===

    /// 插件唯一标识符 (如: "react-query", "vue-query")
    fn id(&self) -> &'static str;

    /// 插件显示名称
    fn name(&self) -> &'static str;

    /// 插件描述
    fn description(&self) -> &'static str;

    /// 插件版本
    fn version(&self) -> &'static str {
        "1.0.0"
    }

    // === 能力声明 ===

    /// 是否支持 Query 功能
    fn supports_query(&self) -> bool {
        false
    }

    /// 是否支持 Mutation 功能
    fn supports_mutation(&self) -> bool {
        false
    }

    /// 支持的文件扩展名
    fn file_extensions(&self) -> Vec<&'static str> {
        vec![".ts"]
    }

    // === 渲染方法 ===

    /// 渲染单个端点
    ///
    /// 这是主要的渲染方法，大多数插件只需实现此方法
    fn render_endpoint(
        &self,
        endpoint: &EndpointItem,
        ctx: &RenderContext
    ) -> Result<Vec<PlannedFile>, String>;

    /// 批量渲染所有端点
    ///
    /// 默认实现逐个调用 render_endpoint
    /// 插件可以覆盖此方法以实现批量优化
    fn render_all(
        &self,
        input: &GeneratorInput,
        ctx: &RenderContext
    ) -> Result<Vec<PlannedFile>, String> {
        let mut files = Vec::new();
        for endpoint in &input.endpoints {
            files.extend(self.render_endpoint(endpoint, ctx)?);
        }
        Ok(files)
    }

    // === 钩子函数 ===

    /// 渲染前钩子
    fn before_render(&self, _ctx: &RenderContext) -> Result<(), String> {
        Ok(())
    }

    /// 渲染后钩子
    fn after_render(
        &self,
        _files: &Vec<PlannedFile>,
        _ctx: &RenderContext
    ) -> Result<Vec<String>, String> {
        Ok(vec![]) // 返回警告列表
    }

    // === 配置 ===

    /// 验证配置
    fn validate_config(&self, _config: &serde_json::Value) -> Result<(), String> {
        Ok(())
    }

    /// 获取默认配置
    fn default_config(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}
```

### 4.2 PluginRegistry

```rust
/// 插件注册表
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn TerminalPlugin>>,
    aliases: HashMap<String, String>, // 别名支持
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    /// 注册插件
    pub fn register(&mut self, plugin: Box<dyn TerminalPlugin>) {
        let id = plugin.id().to_string();
        self.plugins.insert(id.clone(), plugin);
    }

    /// 注册别名
    pub fn register_alias(&mut self, alias: String, target: String) {
        self.aliases.insert(alias, target);
    }

    /// 获取插件（支持别名解析）
    pub fn get(&self, id: &str) -> Option<&dyn TerminalPlugin> {
        let target_id = self.aliases.get(id).unwrap_or(&id.to_string());
        self.plugins.get(target_id).map(|p| p.as_ref())
    }

    /// 列出所有插件
    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins
            .values()
            .map(|p| PluginInfo {
                id: p.id().to_string(),
                name: p.name().to_string(),
                description: p.description().to_string(),
                version: p.version().to_string(),
                supports_query: p.supports_query(),
                supports_mutation: p.supports_mutation(),
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub supports_query: bool,
    pub supports_mutation: bool,
}
```

### 4.3 RenderContext

```rust
/// 渲染上下文
///
/// 提供渲染过程中所需的所有配置和状态信息
#[derive(Debug, Clone)]
pub struct RenderContext {
    /// 模型导入配置
    pub model_import: Option<ModelImportConfig>,

    /// 客户端导入配置
    pub client_import: Option<ClientImportConfig>,

    /// 项目上下文
    pub project: ProjectContext,

    /// 用户自定义配置
    pub user_config: serde_json::Value,

    /// 输出目录
    pub output_dir: PathBuf,

    /// 是否为 dry-run 模式
    pub dry_run: bool,
}

impl RenderContext {
    pub fn new(project: ProjectContext) -> Self {
        Self {
            model_import: None,
            client_import: None,
            project,
            user_config: serde_json::json!({}),
            output_dir: PathBuf::from("./generated"),
            dry_run: false,
        }
    }

    /// 创建子上下文（用于批量渲染）
    pub fn with_config(&self, config: serde_json::Value) -> Self {
        let mut ctx = self.clone();
        ctx.user_config = config;
        ctx
    }
}
```

---

## 5. @aptx 插件实现

### 5.1 AptxFunctionsPlugin

```rust
use crate::pipeline::{EndpointItem, PlannedFile, RenderContext, TerminalPlugin};
use inflector::cases::pascalcase::to_pascal_case;

/// @aptx Functions 终端插件
///
/// 生成使用 @aptx/api-client 的函数式 API 调用代码
pub struct AptxFunctionsPlugin;

impl AptxFunctionsPlugin {
    pub fn new() -> Self {
        Self
    }

    fn get_spec_file_path(endpoint: &EndpointItem) -> String {
        let namespace = endpoint.namespace.join("/");
        format!("spec/endpoints/{namespace}/{}.ts", endpoint.operation_name)
    }

    fn get_function_file_path(endpoint: &EndpointItem) -> String {
        let namespace = endpoint.namespace.join("/");
        format!("functions/api/{namespace}/{}.ts", endpoint.operation_name)
    }

    fn render_spec_file(endpoint: &EndpointItem, ctx: &RenderContext) -> String {
        let builder = format!("build{}Spec", to_pascal_case(&endpoint.operation_name));
        let input_type = normalize_type_ref(&endpoint.input_type_name);
        let model_import_base = get_model_import_base(&ctx.model_import);
        let use_package = should_use_package_import(&ctx.model_import);

        // ... 渲染逻辑
        todo!()
    }

    fn render_function_file(endpoint: &EndpointItem, ctx: &RenderContext) -> String {
        // ... 渲染逻辑
        todo!()
    }
}

impl TerminalPlugin for AptxFunctionsPlugin {
    fn id(&self) -> &'static str {
        "functions"
    }

    fn name(&self) -> &'static str {
        "@aptx Functions"
    }

    fn description(&self) -> &'static str {
        "生成使用 @aptx/api-client 的函数式 API 调用代码"
    }

    fn render_endpoint(
        &self,
        endpoint: &EndpointItem,
        ctx: &RenderContext
    ) -> Result<Vec<PlannedFile>, String> {
        Ok(vec![
            PlannedFile {
                path: Self::get_spec_file_path(endpoint),
                content: self.render_spec_file(endpoint, ctx),
            },
            PlannedFile {
                path: Self::get_function_file_path(endpoint),
                content: self.render_function_file(endpoint, ctx),
            },
        ])
    }
}
```

### 5.2 AptxReactQueryPlugin

```rust
/// @aptx React Query 终端插件
///
/// 生成使用 @aptx/react-query 的 React Query Hooks
pub struct AptxReactQueryPlugin;

impl AptxReactQueryPlugin {
    pub fn new() -> Self {
        Self
    }

    fn get_query_file_path(endpoint: &EndpointItem) -> String {
        let namespace = endpoint.namespace.join("/");
        format!("react-query/{namespace}/{}.query.ts", endpoint.operation_name)
    }

    fn get_mutation_file_path(endpoint: &EndpointItem) -> String {
        let namespace = endpoint.namespace.join("/");
        format!("react-query/{namespace}/{}.mutation.ts", endpoint.operation_name)
    }

    fn render_query_file(endpoint: &EndpointItem, ctx: &RenderContext) -> String {
        // 渲染查询文件
        // 使用 @aptx/api-query-adapter 和 @aptx/react-query
        todo!()
    }

    fn render_mutation_file(endpoint: &EndpointItem, ctx: &RenderContext) -> String {
        // 渲染变更文件
        todo!()
    }
}

impl TerminalPlugin for AptxReactQueryPlugin {
    fn id(&self) -> &'static str {
        "react-query"
    }

    fn name(&self) -> &'static str {
        "@aptx React Query"
    }

    fn description(&self) -> &'static str {
        "生成使用 @aptx/react-query 的 React Query Hooks"
    }

    fn supports_query(&self) -> bool {
        true
    }

    fn supports_mutation(&self) -> bool {
        true
    }

    fn render_endpoint(
        &self,
        endpoint: &EndpointItem,
        ctx: &RenderContext
    ) -> Result<Vec<PlannedFile>, String> {
        let mut files = Vec::new();

        if endpoint.supports_query {
            files.push(PlannedFile {
                path: Self::get_query_file_path(endpoint),
                content: self.render_query_file(endpoint, ctx),
            });
        }

        if endpoint.supports_mutation {
            files.push(PlannedFile {
                path: Self::get_mutation_file_path(endpoint),
                content: self.render_mutation_file(endpoint, ctx),
            });
        }

        Ok(files)
    }
}
```

### 5.3 AptxVueQueryPlugin

```rust
/// @aptx Vue Query 终端插件
///
/// 生成使用 @aptx/vue-query 的 Vue Query Composables
pub struct AptxVueQueryPlugin;

impl AptxVueQueryPlugin {
    pub fn new() -> Self {
        Self
    }

    fn get_query_file_path(endpoint: &EndpointItem) -> String {
        let namespace = endpoint.namespace.join("/");
        format!("vue-query/{namespace}/{}.query.ts", endpoint.operation_name)
    }

    fn get_mutation_file_path(endpoint: &EndpointItem) -> String {
        let namespace = endpoint.namespace.join("/");
        format!("vue-query/{namespace}/{}.mutation.ts", endpoint.operation_name)
    }

    fn render_query_file(endpoint: &EndpointItem, ctx: &RenderContext) -> String {
        // 渲染查询文件
        // 使用 @aptx/api-query-adapter 和 @aptx/vue-query
        todo!()
    }

    fn render_mutation_file(endpoint: &EndpointItem, ctx: &RenderContext) -> String {
        // 渲染变更文件
        todo!()
    }
}

impl TerminalPlugin for AptxVueQueryPlugin {
    fn id(&self) -> &'static str {
        "vue-query"
    }

    fn name(&self) -> &'static str {
        "@aptx Vue Query"
    }

    fn description(&self) -> &'static str {
        "生成使用 @aptx/vue-query 的 Vue Query Composables"
    }

    fn supports_query(&self) -> bool {
        true
    }

    fn supports_mutation(&self) -> bool {
        true
    }

    fn render_endpoint(
        &self,
        endpoint: &EndpointItem,
        ctx: &RenderContext
    ) -> Result<Vec<PlannedFile>, String> {
        let mut files = Vec::new();

        if endpoint.supports_query {
            files.push(PlannedFile {
                path: Self::get_query_file_path(endpoint),
                content: self.render_query_file(endpoint, ctx),
            });
        }

        if endpoint.supports_mutation {
            files.push(PlannedFile {
                path: Self::get_mutation_file_path(endpoint),
                content: self.render_mutation_file(endpoint, ctx),
            });
        }

        Ok(files)
    }
}
```

### 5.4 标准终端插件

```rust
/// 标准 Axios TypeScript 终端插件
///
/// 生成使用 tsyringe 和 axios 的服务类
pub struct StandardAxiosTsPlugin;

impl TerminalPlugin for StandardAxiosTsPlugin {
    fn id(&self) -> &'static str {
        "axios-ts"
    }

    fn name(&self) -> &'static str {
        "Axios TypeScript"
    }

    fn description(&self) -> &'static str {
        "生成使用 tsyringe 和 axios 的 TypeScript 服务类"
    }

    fn render_endpoint(
        &self,
        endpoint: &EndpointItem,
        ctx: &RenderContext
    ) -> Result<Vec<PlannedFile>, String> {
        // 不依赖 @aptx 包的纯 axios 实现
        todo!()
    }
}
```

---

## 6. 迁移步骤

### 步骤 1：定义插件接口

**目标**：在 `swagger_gen` 中创建插件基础设施

**文件**：`crates/swagger_gen/src/pipeline/plugin.rs`

```rust
// plugin.rs
pub trait TerminalPlugin: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn render_endpoint(&self, endpoint: &EndpointItem, ctx: &RenderContext)
        -> Result<Vec<PlannedFile>, String>;
    fn render_all(&self, input: &GeneratorInput, ctx: &RenderContext)
        -> Result<Vec<PlannedFile>, String>;
}

pub struct RenderContext {
    pub model_import: Option<ModelImportConfig>,
    pub client_import: Option<ClientImportConfig>,
    pub project: ProjectContext,
    pub user_config: serde_json::Value,
    pub output_dir: std::path::PathBuf,
    pub dry_run: bool,
}

pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn TerminalPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, plugin: Box<dyn TerminalPlugin>);
    pub fn get(&self, id: &str) -> Option<&dyn TerminalPlugin>;
    pub fn list(&self) -> Vec<PluginInfo>;
}
```

### 步骤 2：创建 @aptx 插件

**目标**：将现有的 `@aptx` 相关代码迁移到独立插件

**文件结构**：
```
crates/swagger_gen/src/pipeline/plugins/
├── mod.rs
├── aptx.rs          # AptxFunctionsPlugin, AptxReactQueryPlugin, AptxVueQueryPlugin
└── standard.rs      # StandardAxiosTsPlugin, StandardAxiosJsPlugin, StandardUniAppPlugin
```

**迁移内容**：
- `FunctionsRenderer` → `AptxFunctionsPlugin`
- `ReactQueryRenderer` → `AptxReactQueryPlugin`
- `VueQueryRenderer` → `AptxVueQueryPlugin`

### 步骤 3：修改核心使用插件

**目标**：重构 `renderer.rs` 使用插件系统

**修改**：`crates/swagger_gen/src/pipeline/renderer.rs`

```rust
// 修改前
impl Renderer for ReactQueryRenderer {
    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String> {
        render_query_terminal(input, QueryTerminal::React)  // 硬编码 @aptx
    }
}

// 修改后
pub struct PluginRenderer {
    registry: Arc<PluginRegistry>,
}

impl PluginRenderer {
    pub fn new(registry: Arc<PluginRegistry>) -> Self {
        Self { registry }
    }
}

impl Renderer for PluginRenderer {
    fn id(&self) -> &'static str {
        "plugin"
    }

    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String> {
        let mut all_files = Vec::new();
        let mut all_warnings = Vec::new();

        // 从配置获取要使用的终端列表
        for terminal_id in &input.project.terminals {
            if let Some(plugin) = self.registry.get(terminal_id) {
                let ctx = RenderContext::new(input.project.clone());
                let files = plugin.render_all(input, &ctx)?;
                all_files.extend(files);

                // 执行 after_render 钩子
                let warnings = plugin.after_render(&all_files, &ctx)?;
                all_warnings.extend(warnings);
            } else {
                return Err(format!("Terminal not found: {terminal_id}"));
            }
        }

        Ok(RenderOutput {
            files: all_files,
            warnings: all_warnings,
        })
    }
}
```

### 步骤 4：测试验证

**测试矩阵**：

| 测试项 | 描述 | 预期结果 |
|--------|------|----------|
| 现有终端 | 运行现有所有终端 | 输出与迁移前一致 |
| 新增终端 | 添加自定义插件 | 成功注册和运行 |
| 错误处理 | 不存在的终端 | 返回清晰错误信息 |
| 配置切换 | 切换客户端导入模式 | 生成正确的导入语句 |
| 并发执行 | 多个终端同时运行 | 无竞态条件 |

---

## 7. 向后兼容策略

### 7.1 API 兼容性

#### 7.1.1 保留现有 Renderer 实现

```rust
// 保留现有的 Renderer trait 和实现
pub trait Renderer {
    fn id(&self) -> &'static str;
    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String>;
}

// 现有实现标记为 deprecated 但保持功能
#[deprecated(since = "2.0.0", note = "Use AptxFunctionsPlugin instead")]
impl Renderer for FunctionsRenderer {
    // ... 现有实现
}
```

#### 7.1.2 提供适配器

```rust
/// 将 TerminalPlugin 适配为 Renderer
pub struct PluginAdapter {
    plugin: Box<dyn TerminalPlugin>,
    terminal_id: String,
}

impl Renderer for PluginAdapter {
    fn id(&self) -> &'static str {
        &self.terminal_id
    }

    fn render(&self, input: &GeneratorInput) -> Result<RenderOutput, String> {
        let ctx = RenderContext::new(input.project.clone());
        let files = self.plugin.render_all(input, &ctx)?;
        Ok(RenderOutput {
            files,
            warnings: vec![],
        })
    }
}
```

### 7.2 配置兼容性

#### 7.2.1 CLI 参数兼容

```bash
# 现有用法继续有效
aptx-ft terminal:codegen --terminal react-query --output ./output

# 新增插件式用法
aptx-ft codegen run --terminals react-query,vue-query --output ./output
```

#### 7.2.2 配置文件兼容

```typescript
// aptx-ft.config.ts
const config: APTXFtConfig = {
  input: "./openapi.json",
  codegen: {
    outputRoot: "./generated",
    terminals: [
      "react-query",  // 自动映射到 AptxReactQueryPlugin
      "vue-query",   // 自动映射到 AptxVueQueryPlugin
      "functions",   // 自动映射到 AptxFunctionsPlugin
    ],
  },
};
```

---

## 8. 文件结构变更

### 8.1 当前结构

```
crates/swagger_gen/src/pipeline/
├── mod.rs
├── model.rs
├── renderer.rs       # 硬编码 @aptx 引用
├── orchestrator.rs
├── parser.rs
├── transform.rs
└── writer.rs
```

### 8.2 迁移后结构

```
crates/swagger_gen/src/pipeline/
├── mod.rs
├── model.rs
├── plugin.rs           # 新增：插件核心接口
├── renderer.rs         # 重构：使用插件系统
├── plugins/           # 新增：插件实现目录
│   ├── mod.rs
│   ├── aptx.rs        # @aptx 系列插件
│   └── standard.rs    # 标准插件
├── orchestrator.rs
├── parser.rs
├── transform.rs
└── writer.rs
```

### 8.3 新增文件清单

| 文件 | 行数估计 | 说明 |
|------|----------|------|
| `plugin.rs` | ~200 | 插件核心接口定义 |
| `plugins/mod.rs` | ~50 | 插件模块导出 |
| `plugins/aptx.rs` | ~400 | @aptx 插件实现 |
| `plugins/standard.rs` | ~300 | 标准插件实现 |

---

## 9. 示例代码

### 9.1 使用插件系统

```rust
use swagger_gen::pipeline::{
    PluginRegistry, RenderContext, AptxReactQueryPlugin,
    AptxVueQueryPlugin, AptxFunctionsPlugin,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建注册表
    let mut registry = PluginRegistry::new();

    // 注册 @aptx 插件
    registry.register(Box::new(AptxReactQueryPlugin::new()));
    registry.register(Box::new(AptxVueQueryPlugin::new()));
    registry.register(Box::new(AptxFunctionsPlugin::new()));

    // 注册标准插件
    registry.register(Box::new(StandardAxiosTsPlugin::new()));

    // 使用插件渲染
    let input = parse_openapi("openapi.json")?;
    let ctx = RenderContext::new(input.project.clone());

    let plugin = registry.get("react-query")
        .ok_or("Plugin not found")?;

    let files = plugin.render_all(&input, &ctx)?;

    // 写入文件
    for file in files {
        std::fs::write(
            format!("generated/{}", file.path),
            file.content
        )?;
    }

    Ok(())
}
```

### 9.2 创建自定义插件

```rust
/// 自定义 Axios 终端插件
pub struct MyCustomAxiosPlugin;

impl TerminalPlugin for MyCustomAxiosPlugin {
    fn id(&self) -> &'static str {
        "my-axios"
    }

    fn name(&self) -> &'static str {
        "My Custom Axios"
    }

    fn description(&self) -> &'static str {
        "自定义 Axios 实现，使用特定的错误处理"
    }

    fn render_endpoint(
        &self,
        endpoint: &EndpointItem,
        ctx: &RenderContext
    ) -> Result<Vec<PlannedFile>, String> {
        let content = format!(
            "import axios from 'axios';\n\n\
             export async function {}(input: any) {{\n\
             // 自定义实现\n\
             return axios.{}('{}', input);\n\
             }}",
            endpoint.operation_name,
            endpoint.method.to_lowercase(),
            endpoint.path
        );

        Ok(vec![PlannedFile {
            path: format!("{}.ts", endpoint.operation_name),
            content,
        }])
    }
}

// 注册自定义插件
registry.register(Box::new(MyCustomAxiosPlugin));
```

---

## 10. 时间线和里程碑

### 阶段 1：基础设施（1-2 周）

- [ ] 定义 `TerminalPlugin` trait
- [ ] 实现 `PluginRegistry`
- [ ] 定义 `RenderContext`
- [ ] 编写单元测试

### 阶段 2：@aptx 插件迁移（2-3 周）

- [ ] 实现 `AptxFunctionsPlugin`
- [ ] 实现 `AptxReactQueryPlugin`
- [ ] 实现 `AptxVueQueryPlugin`
- [ ] 迁移测试用例

### 阶段 3：核心重构（2-3 周）

- [ ] 重构 `renderer.rs` 使用插件系统
- [ ] 更新 `orchestrator.rs` 调用流程
- [ ] 集成测试

### 阶段 4：兼容性保证（1-2 周）

- [ ] 实现 `PluginAdapter`
- [ ] 保留现有 API
- [ ] 文档更新

### 阶段 5：标准插件（可选，1-2 周）

- [ ] 实现 `StandardAxiosTsPlugin`
- [ ] 实现 `StandardAxiosJsPlugin`
- [ ] 实现 `StandardUniAppPlugin`

---

## 附录

### A. 相关文档

- [最终代码生成器架构](./final-codegen-architecture.md)
- [代码生成器使用指南](./codegen-guide.md)
- [三层架构设计](../../aptx-root/docs/three-layer-architecture.md)

### B. 迁移检查清单

- [ ] 所有 @aptx 引用已移出核心代码
- [ ] 插件接口设计完整且稳定
- [ ] 现有终端输出与迁移前一致
- [ ] 单元测试覆盖率不低于 80%
- [ ] 集成测试通过
- [ ] 文档更新完成
- [ ] 向后兼容性验证通过

### C. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| API 变更导致用户代码中断 | 高 | 提供适配器层，保留现有 API |
| 性能回退 | 中 | 基准测试，优化关键路径 |
| 插件生态系统碎片化 | 低 | 提供清晰的插件开发指南 |
| 迁移时间超预期 | 中 | 分阶段实施，每阶段独立可用 |
