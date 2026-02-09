# Frontend TK 下一代生成器架构（精简版）

状态：Proposal  
日期：2026-02-09  
适用仓库：`frontend_tk_rs`

## 1. 目标

1. 完全切换到新 CLI（不兼容旧 `gen --service-mode/--service-output`）。
2. 生成器可按需扩展：新增 terminal / layout / transform 不改核心。
3. 大规模 OpenAPI 下保持高性能（并行、增量、低 IO）。
4. 插件命令可被 `aptx --help` 自动发现并展示。
5. 模板层只消费稳定 IR，不耦合 OpenAPI 原始结构。

## 2. 架构总览

```txt
CLI (Node)
  -> Native Core (Rust, N-API)
    -> Command Registry V2 (built-in + plugin)
      -> Command Runtime
        -> Parser (OpenAPI -> Canonical IR)
        -> Transform Pipeline
        -> Renderer Pipeline
        -> File Planner + Writer
```

分层职责：

1. `parser`：OpenAPI -> `GeneratorInput`。
2. `transform`：命名、分组、路由、扩展字段注入。
3. `renderer`：按 terminal 生成 `PlannedFile[]`。
4. `layout`：目录与文件落盘策略。
5. `writer`：冲突检测、hash 跳过、原子写入。

## 3. CLI 与命令系统

## 3.1 新 CLI 形态

```bash
aptx <command> [subcommand] [options]
```

示例：

1. `aptx codegen run -c codegen.config.ts`
2. `aptx codegen list-terminals`
3. `aptx doctor`
4. `aptx plugin list`

## 3.2 Command Registry V2（帮助可发现）

命令注册从“名字 + 回调”升级为“元数据 + 处理器”。

```rust
pub trait CommandHandler: Send + Sync {
  fn descriptor(&self) -> CommandDescriptor;
  fn run(&self, ctx: CommandContext) -> Result<(), CommandError>;
}
```

`CommandDescriptor` 至少包含：

1. `name/summary/description`
2. `options`
3. `examples`
4. `plugin_name/plugin_version`

帮助系统统一读取 `registry.list_descriptors()`：

1. `aptx --help` 展示全部命令（内置 + 插件）。
2. `aptx <command> --help` 展示该命令参数、示例、来源插件。
3. `aptx --json-help` 输出机器可读帮助。

## 4. 插件模型（合并版）

## 4.1 双通道插件

1. `native-rust`：性能优先。
2. `script-js`：灵活优先。
3. `wasm`（可选）：性能与隔离折中。

## 4.2 统一契约

三类插件都实现同一协议：

1. `descriptor()`：命令与帮助元数据。
2. `capabilities()`：声明提供的能力（command/renderer/transform/layout）。
3. `run()` 或 `render()`：执行逻辑。
4. `apiVersion`：版本握手。

核心不关心插件内部语言，只认协议。

## 4.3 统一输入输出

1. 输入：标准 IR（`GeneratorInput` + 扩展字段）。
2. 输出：`PlannedFile[]`。
3. 所有 IO 统一由 core writer 处理。

## 5. 生成器契约（IR First）

以 `docs/codegen-template-contract-draft.md` 为模板契约基线，并扩展：

1. `schemaGraph`：类型依赖图。
2. `security`：鉴权元信息。
3. `vendorExtensions`：保留 `x-*` 字段。
4. `fingerprint`：endpoint 级 hash（增量构建）。

终端通过能力声明驱动路由，避免写死 GET/POST 分支。

## 6. 性能策略

1. 并行：terminal 级 + endpoint 分片级。
2. 缓存键：`specHash + pluginVersion + configHash + endpointFingerprint`。
3. 跳写：内容 hash 未变化则不落盘。
4. 写入：临时文件 + rename 原子落盘。
5. 观测：`--dry-run`、`--profile`、`--report-json`。

## 7. 配置模型（新）

```ts
export default defineConfig({
  input: "./openapi.json",
  outputRoot: "./generated",
  terminals: [
    { id: "functions", layout: "contract-v1" },
    { id: "react-query", layout: "contract-v1" },
    { id: "axios-ts", layout: "flat-service" }
  ],
  performance: {
    concurrency: "auto",
    cache: true,
    format: "safe"
  }
})
```

## 8. 对当前仓库的直接改造点

1. `crates/node_binding_plugin/src/command.rs`：改为 V2 命令元数据注册。
2. `crates/node_binding/src/bootstrap.rs`：插件加载与命令冲突处理。
3. `crates/node_binding/src/lib.rs`：导出 help tree 与执行入口。
4. `crates/swagger_gen`：拆分 parser/transform/renderer/layout/writer。
5. `packages/frontend-tk-cli`：动态命令树驱动 CLI 与 help。

## 9. 风险与边界

1. 旧 CLI 不兼容：需明确迁移窗口与提示。
2. 双通道插件会增加 runtime 复杂度：必须先定协议再扩展能力。
3. 脚本插件安全隔离需要尽早纳入（权限、超时、资源限制）。

## 10. 关联文档

1. 模板与输出契约：`docs/codegen-template-contract-draft.md`
2. 分阶段执行计划：`docs/codegen-execution-plan.md`
