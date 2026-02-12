## 使用命令

`aptx-ft` 用于根据 OpenAPI 输入执行代码生成。

### codegen run

执行生成主流程：

```bash
aptx-ft codegen run -c ./aptx-ft.config.ts
```

### codegen list-terminals

查看当前内置 terminal：

```bash
aptx-ft codegen list-terminals
```

### doctor

检查运行环境、binding 与命令注册状态：

```bash
aptx-ft doctor
```

### model gen

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

### model ir

导出模型中间层（Model IR）JSON：

```bash
aptx-ft model ir --output ./tmp/model-ir.json
```
