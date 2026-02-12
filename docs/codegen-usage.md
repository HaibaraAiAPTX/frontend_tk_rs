# Frontend TK 代码生成器使用说明（最终版）

状态：Active  
日期：2026-02-10

## 1. 前置条件

1. 已安装 Node.js（建议与仓库当前版本一致）。
2. 已安装 pnpm。
3. 在仓库根目录执行命令。

## 2. CLI 概览

可执行命令：`aptx-ft`

一级命令：

1. `codegen run`
2. `codegen list-terminals`
3. `model gen`
4. `model ir`
5. `doctor`
6. `plugin list`
7. `<native/plugin command>`（由命令注册表动态扩展）

查看帮助：

```bash
aptx-ft --help
aptx-ft codegen run --help
aptx-ft doctor --help
```

## 3. 配置文件

默认配置文件：`./aptx-ft.config.ts`  
可通过 `-c` 或 `--config` 指定。

示例：

```ts
import type { APTXFtConfig } from "@aptx/frontend-tk-types";

const config: APTXFtConfig = {
  input: "./openapi.json",
  plugin: [
    // native plugin (.dll/.so/.dylib)
    // "./plugins/native-plugin.dll",
    // script plugin (.js/.cjs/.mjs)
    // "./plugins/custom-renderer.cjs",
  ],
  codegen: {
    outputRoot: "./generated",
    terminals: [
      "axios-ts",
      { id: "react-query", output: "./generated/rq" },
      "vue-query",
    ],
  },
  performance: {
    concurrency: "auto",
    cache: true,
  },
  scriptPluginPolicy: {
    timeoutMs: 30_000,
    maxWriteFiles: 10_000,
    maxWriteBytes: 100 * 1024 * 1024,
    maxHeapMb: 1024,
  },
};

export default config;
```

## 4. 常用命令

## 4.1 执行生成

```bash
aptx-ft codegen run -c ./aptx-ft.config.ts
```

可用参数：

1. `--dry-run`：只生成计划，不落盘。
2. `--profile`：输出耗时统计。
3. `--report-json <file>`：输出 JSON 报告。
4. `--concurrency <auto|N>`：覆盖并发度。
5. `--cache <true|false>`：覆盖缓存开关。
6. `-i, --input <path|url>`：覆盖输入 OpenAPI。
7. `-p, --plugin <paths...>`：追加插件路径。

## 4.2 查看 terminal 支持情况

```bash
aptx-ft codegen list-terminals
```

当前内置 terminal：

1. `axios-ts`
2. `axios-js`
3. `uniapp`
4. `functions`
5. `react-query`
6. `vue-query`

## 4.3 运行健康检查

```bash
aptx-ft doctor
```

输出包括：

1. Node 版本
2. Binding 是否可用
3. 已注册命令列表
4. Script 插件加载数量

## 4.4 查看插件提供方

```bash
aptx-ft plugin list
```

## 4.5 生成模型声明

```bash
aptx-ft model gen --output ./generated/models
```

生成模块化模型（`export interface` + `.ts`）：

```bash
aptx-ft model gen --output ./generated/models --style module
```

仅生成指定 schema：

```bash
aptx-ft model gen --output ./generated/models --name Order --name User
```

导出模型中间层（Model IR）：

```bash
aptx-ft model ir --output ./tmp/model-ir.json
```

## 5. 直接调用内置 native 命令

## 5.1 导出 IR 快照

```bash
aptx-ft ir:snapshot -i ./openapi.json --output ./tmp/ir.json
```

## 5.2 单 terminal 生成（native）

```bash
aptx-ft terminal:codegen -i ./openapi.json --terminal axios-ts --output ./generated/services/axios-ts
```

## 6. Script 插件开发最小示例

```js
/** @type {import("node:module")} */
module.exports = {
  apiVersion: "1",
  pluginName: "demo-script-plugin",
  pluginVersion: "0.1.0",
  commands: [
    {
      name: "demo:echo",
      summary: "print endpoint count",
      run: async ({ getIrSnapshot }) => {
        const ir = await getIrSnapshot();
        console.log(`endpoint count: ${ir.endpoints.length}`);
      },
    },
  ],
  renderers: [
    {
      id: "demo-terminal",
      render: async ({ ir, writeFile }) => {
        writeFile(
          "index.ts",
          `export const endpointCount = ${ir.endpoints.length};\n`,
        );
      },
    },
  ],
};
```

在配置中注册：

```ts
export default {
  input: "./openapi.json",
  plugin: ["./plugins/demo-plugin.cjs"],
  codegen: {
    outputRoot: "./generated",
    terminals: ["demo-terminal"],
  },
};
```

## 7. 故障排查

1. 报错 `input is required`：补充 `-i` 或配置 `input`。
2. 报错 `codegen config is required`：补充 `codegen` 配置段。
3. 报错 `Terminal ... not supported`：
   - terminal 不是内置 terminal。
   - 且未在 script 插件中注册同名 renderer。
4. script 超时或写入超限：调整 `scriptPluginPolicy`，或拆分生成任务。

## 8. 产物与缓存

1. 默认输出根目录：`./generated`。
2. 缓存文件：`<outputRoot>/.aptx-cache/run-cache.json`。
3. 缓存命中前提：输入与关键配置哈希一致且输出目录存在。
