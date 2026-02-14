## 使用命令

`aptx-ft` 用于根据 OpenAPI 输入执行代码生成。

### CLI 命令

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

### 所有命令一览

所有命令通过 `aptx-ft <namespace> <command>` 的方式调用：

| 命名空间 | 命令 | 描述 |
|----------|------|------|
| aptx | `functions` | 生成 @aptx 函数式 API 调用 |
| aptx | `react-query` | 生成 React Query hooks |
| aptx | `vue-query` | 生成 Vue Query composables |
| model | `gen` | 生成 TypeScript 模型声明 |
| model | `ir` | 导出模型 IR 快照 JSON |
| model | `enum-plan` | 导出枚举增强计划 JSON |
| model | `enum-apply` | 应用枚举补丁并生成模型 |
| input | `download` | 从 URL 下载 OpenAPI JSON |

查看命令详细帮助：

```bash
aptx-ft aptx functions --help
aptx-ft model gen --help
aptx-ft model ir --help
aptx-ft model enum-plan --help
aptx-ft model enum-apply --help
aptx-ft input download --help
```
