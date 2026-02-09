# 下一代代码生成器执行计划（TODO）

状态：Execution TODO  
日期：2026-02-09  
前置文档：`docs/next-gen-codegen-architecture.md`

使用方式：每完成一项，将 `- [ ]` 改为 `- [x]`。

## M1 命令系统 V2（先不改生成器）

### 目标

- [x] CLI 可动态展示内置与插件命令帮助。

### 交付

- [x] 定义 `CommandDescriptor`。
- [x] 定义 `OptionDescriptor`。
- [x] 定义 `CommandHandler` trait。
- [x] `CommandRegistry` 支持 `register/list/execute`。
- [x] N-API 导出 `get_help_tree()`（或等价接口）。
- [x] CLI 改为 descriptor 驱动 help 渲染。

### 验收

- [x] `aptx --help` 可展示插件命令。
- [x] `aptx <plugin-cmd> --help` 可展示参数与示例。

## M2 新 CLI 切换

### 目标

- [x] 完成不兼容切换，统一新命令入口。

### 交付

- [x] 提供 `codegen run`。
- [x] 提供 `codegen list-terminals`。
- [x] 提供 `doctor`。
- [x] 提供 `plugin list`。
- [x] 新配置结构：`input/outputRoot/terminals/performance`。
- [x] 旧参数输出明确错误与迁移提示。

### 验收

- [x] 新命令链路完整可用。
- [x] 旧参数不再执行旧逻辑。

## M3 IR 与 pipeline 骨架

### 目标

- [x] 解耦 OpenAPI 与模板。

### 交付

- [x] 新增 `parser`：OpenAPI -> `GeneratorInput`。
- [x] 新增 `transform`：命名/路由/扩展字段。
- [x] 新增 `renderer` 抽象接口。
- [x] 新增 `layout` 抽象接口。
- [x] 新增 `writer` 抽象接口。
- [x] 建立 orchestrator 串联 pipeline。

### 验收

- [x] 可输出 IR 快照。
- [x] pipeline 空跑可完成计划与报告。

## M4 最小可用生成（functions）

### 目标

- [x] 跑通第一条真实生成链路。

### 交付

- [x] 实现 `functions` renderer（contract-v1 目录规则）。
- [x] `writer` 支持 hash 跳过。
- [x] `writer` 支持原子写入。
- [x] 增加最小模板快照测试。

### 验收

- [x] one-endpoint-per-file 生效。
- [x] 固定输入生成结果稳定。

## M5 query 终端扩展

### 目标

- [x] 补齐 `react-query` 与 `vue-query` 终端。

### 交付

- [x] 实现 query renderer。
- [x] 实现 mutation renderer。
- [x] 实现 `signal` 透传契约。
- [x] 实现 `meta` 透传契约。
- [x] 实现稳定 `queryKey` 契约。
- [x] 增加终端级快照测试。
- [x] 增加 `tsc --noEmit` 验证。

### 验收

- [x] 两类终端可同时生成。
- [x] 终端之间无互相依赖。

## M6 双通道插件（native-rust + script-js）

### 目标

- [x] 同时支持高性能与高灵活模板扩展。

### 交付

- [x] 支持 script-js command 插件加载。
- [x] 支持 script-js renderer 插件加载。
- [x] 插件握手字段：`apiVersion/pluginName/pluginVersion`。
- [x] 插件错误隔离与上下文报错。

### 验收

- [x] JS 插件无需 Rust 编译即可被 `--help` 发现并执行。
- [ ] Rust/JS 插件共享同一 IR 契约。

说明：
- 当前 script-js 插件已完成命令/renderer 双通道接入，IR 共享将作为下一步子任务推进（优先通过 binding 暴露 IR snapshot 接口后接入）。

## M7 性能与可观测性

### 目标

- [ ] 确保规模化可用。

### 交付

- [ ] 并行渲染与分片执行。
- [ ] 增量缓存机制。
- [ ] `--dry-run`。
- [ ] `--profile`。
- [ ] `--report-json`。

说明：
- `swagger_gen` pipeline 已补充 `ExecutionPlan.metrics` 与 `build_report_json()`，CLI 参数层 (`--profile/--report-json`) 仍待接入。

### 验收

- [ ] 10k endpoint 二次生成耗时显著低于首次（目标 <20%）。
- [ ] 报告可定位到命令、插件、endpoint、文件。

## 实施顺序（建议）

- [x] 先完成 M1 + M2，冻结命令元数据协议。
- [x] 再完成 M3 + M4，打通最小生成闭环。
- [x] 然后完成 M5 + M6，扩展终端与插件。
- [ ] 最后完成 M7，集中做性能与可观测。

## 风险与缓解

- [ ] M1 完成后冻结 `CommandDescriptor` 字段，避免协议漂移。
- [ ] 脚本插件默认限制权限、执行时间、内存。
- [x] 每个里程碑结束后同步更新文档，避免实现与文档偏移。

## 完成定义（DoD）

- [x] 新 CLI 可通过 `--help` 发现全部命令（含插件）。
- [ ] 生成器完成 IR-first 切换，模板层不再读取 OpenAPI 原始结构。
- [x] 同时具备 native-rust 与 script-js 插件能力。
- [ ] 具备可量化性能指标与执行报告。
