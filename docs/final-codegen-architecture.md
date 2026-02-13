# Frontend TK 代码生成器功能架构（最终版）

状态：Active  
日期：2026-02-10  
适用仓库：`frontend_tk_rs`

## 1. 目标与边界

1. 仅保留新 CLI 体系，不兼容旧 `gen` 模式。
2. 以 IR-first 作为统一生成输入，模板层不直接读取 OpenAPI 原始结构。
3. 支持两类扩展能力：
   - native（Rust）：高性能内置或插件命令。
   - script（JS）：无需 Rust 编译的灵活扩展。
4. 命令注册可被 `--help` 自动发现与展示。

## 2. 总体分层

```txt
aptx-ft (Node CLI)
  -> @aptx/frontend-tk-binding (N-API)
    -> Command Registry V2
      -> Built-in Commands / Native Plugins
      -> IR Pipeline (swagger_gen)
        -> parser -> transform -> renderer -> writer
  -> Script Plugin Runtime (JS)
    -> script commands / script renderers
```

分层职责：

1. CLI 层：参数解析、配置加载、并发调度、缓存命中控制、报告输出。
2. Binding 层：命令注册、帮助树导出、命令执行入口。
3. Pipeline 层：OpenAPI -> 统一 IR -> 各 terminal 产物。
4. Script Runtime：在同一 IR 契约下执行 JS 命令与渲染器。

## 3. 命令系统 V2

命令统一由 `CommandDescriptor` 驱动：

1. 基础信息：`name`、`summary`、`description`。
2. 参数信息：`options`（含 long/short/valueName/required 等）。
3. 示例：`examples`。
4. 来源：`pluginName`、`pluginVersion`。

当前内置命令：

1. **IR 相关**
   - `ir:snapshot`：导出 IR JSON 快照。
2. **Terminal 生成**
   - `terminal:codegen`：按单 terminal 执行内置生成。
3. **Model 生成**
   - `model:gen`：生成 TypeScript 类型声明。
   - `model:ir`：导出模型 IR 快照。
   - `model:enum-plan`：导出枚举增强计划 JSON。
   - `model:enum-apply`：应用枚举补丁并生成模型。
4. **插件命令**（通过 native plugin 提供）
   - `materal:antd-init`：生成 Ant Design 脚手架。
   - `materal:enum-patch`：获取 Materal 枚举值并输出补丁。

帮助发现机制：

1. `aptx-ft --help`：展示 CLI 一级命令。
2. `aptx-ft <native-cmd> --help`：由 descriptor 渲染参数与示例。
3. `aptx-ft plugin list` / `aptx-ft doctor`：查看命令来源与注册状态。

## 4. IR 与生成契约

IR 最小核心结构：

1. `project`：项目上下文（包名、basePath、terminals、重试归属）。
2. `endpoints[]`：端点清单（namespace、operation、method、path、输入输出类型、query/path/body 字段、query/mutation 能力、deprecated 标记）。

关键约束：

1. renderer 仅消费 IR，不读取 OpenAPI 原始结构。
2. writer 统一负责写盘，支持 hash 跳过与原子落盘。
3. terminal 之间禁止耦合导入。

## 5. Terminal 体系

当前内置 terminal：

1. `axios-ts`
2. `axios-js`
3. `uniapp`
4. `functions`
5. `react-query`
6. `vue-query`

执行路径：

1. CLI `codegen run` 将 terminal 拆分为 built-in 与 script 两组。
2. built-in terminal 走 `terminal:codegen`（Rust pipeline）。
3. script terminal 走 JS renderer（`writeFile/writeFiles` 受策略限制）。

## 6. 插件模型（性能与灵活性并存）

## 6.1 Native 插件

1. 形态：动态库（Rust 编译产物）。
2. 优点：性能高，适合重计算和深度集成。
3. 成本：开发与分发门槛更高。

## 6.2 Script 插件

1. 形态：`.js/.cjs/.mjs`。
2. 协议：`apiVersion/pluginName/pluginVersion` 必填。
3. 能力：可注册 `commands[]` 与 `renderers[]`。
4. 优点：无需 Rust 编译，迭代快，适合业务定制模板。

## 6.3 安全与资源策略

由 `scriptPluginPolicy` 控制：

1. `timeoutMs`
2. `maxWriteFiles`
3. `maxWriteBytes`
4. `maxHeapMb`

并且限制 script renderer 只能写入 terminal 输出根目录内。

## 7. 性能与可观测

1. 并发：`codegen run --concurrency auto|N`。
2. 缓存：按输入与配置哈希命中，跳过整次生成。
3. Dry Run：`--dry-run` 只构建计划不写文件。
4. Profiling：`--profile` 输出总耗时与 terminal 级耗时。
5. 报告：`--report-json` 输出结构化执行报告（含文件与 endpoint 映射）。

## 8. 关键实现位置

1. CLI 主入口：`packages/frontend-tk-cli/src/index.ts`
2. Built-in 命令注册：`crates/node_binding/src/built_in/mod.rs`
3. Built-in terminal 生成：`crates/node_binding/src/built_in/terminal_codegen.rs`
4. IR pipeline：`crates/swagger_gen/src/pipeline/*`

## 9. 演进规则

1. 新增命令必须提供 descriptor，确保帮助可发现。
2. 新增 terminal 优先实现 renderer，不改 CLI 主流程。
3. 新增插件能力必须先定义稳定契约，再开放实现。
4. 文档维护：
   - 本文件（架构设计）与 `docs/codegen-guide.md`（使用指南）为唯二核心文档
   - 避免创建重复文档，保持单一信息源
