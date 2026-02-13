## 使用命令

`aptx-ft` 用于根据 OpenAPI 输入执行代码生成。

### CLI 命令

#### codegen run

执行生成主流程：

```bash
aptx-ft codegen run --input ./openapi.json --output ./generated
```

#### codegen list-terminals

查看当前内置 terminal：

```bash
aptx-ft codegen list-terminals
```

#### doctor

检查运行环境、binding 与命令注册状态：

```bash
aptx-ft doctor
```

#### model gen

生成 TypeScript 模型文件（基于 OpenAPI `components/schemas`）：

```bash
aptx-ft model gen --output ./generated/models
```

生成 `export interface` 风格（`.ts`）：

```bash
aptx-ft model gen --output ./generated/models --style module
```

只生成指定模型：

```bash
aptx-ft model gen --output ./generated/models --name Order --name User
```

#### model ir

导出模型中间层（Model IR） JSON：

```bash
aptx-ft model ir --output ./tmp/model-ir.json
```

#### model enum-plan

从模型 IR 导出枚举增强计划 JSON：

```bash
aptx-ft model enum-plan --output ./tmp/enum-plan.json
```

#### model enum-apply

应用枚举补丁文件并生成模型文件：

```bash
aptx-ft model enum-apply --patch ./tmp/enum-patch.json --output ./generated/models
```

指定冲突策略：

```bash
aptx-ft model enum-apply --patch ./tmp/enum-patch.json --output ./generated/models --conflict-policy patch-first
```

#### input download

从 URL 下载 OpenAPI JSON 到本地文件：

```bash
aptx-ft input download --url https://example.com/openapi.json --output ./openapi.json
```

#### plugin list

列出已加载的命令提供者（内置 + 插件）：

```bash
aptx-ft plugin list
```

### 原生命令 (Native Commands)

原生命令通过 `aptx-ft <command> [args...]` 的方式直接调用：

| 命令 | 描述 |
|------|------|
| `terminal:codegen` | 从 OpenAPI 输入生成单个内置 terminal 的输出 |
| `model:gen` | 从 OpenAPI schemas 生成 TypeScript 模型声明 |
| `model:ir` | 从 OpenAPI schemas 导出模型 IR 快照 JSON |
| `model:enum-plan` | 从模型 IR 导出枚举增强计划 JSON |
| `model:enum-apply` | 应用枚举补丁文件并生成模型文件 |
| `ir:snapshot` | 从 OpenAPI 输入导出 IR 快照 JSON |

查看原生命令的详细帮助：

```bash
aptx-ft terminal:codegen --help
aptx-ft model:gen --help
aptx-ft model:ir --help
aptx-ft model:enum-plan --help
aptx-ft model:enum-apply --help
aptx-ft ir:snapshot --help
```
