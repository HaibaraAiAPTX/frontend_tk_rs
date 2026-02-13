# Frontend TK 代码生成器使用指南

状态：Active
日期：2026-02-13
适用仓库：`frontend_tk_rs`

## 1. 概述

### 1.1 简介

Frontend TK 是一个基于 OpenAPI 规范的代码生成器，专为前端项目设计。它采用 IR-first（中间表示优先）架构，支持多种输出终端，并提供灵活的插件扩展机制。

### 1.2 核心特性

- **IR-first 架构**：统一的中间表示，确保不同终端输出的一致性
- **多终端支持**：内置多种终端（axios-ts、react-query 等），支持自定义终端
- **双插件模型**：支持 Native（Rust）和 Script（JavaScript）两种插件类型
- **增量缓存**：基于输入哈希的智能缓存，加速重复生成
- **并发生成**：支持多终端并发生成，提升大型项目生成速度
- **类型安全**：完整的 TypeScript 类型支持

### 1.3 安装

```bash
pnpm add -D @aptx/frontend-tk-cli
```

### 1.4 快速开始

运行生成命令：

```bash
aptx-ft -i ./openapi.json codegen run --terminals axios-ts react-query --output ./generated
```

## 2. CLI 命令完整列表

### 2.1 `codegen run` - 主生成命令

执行代码生成。

#### 语法

```bash
aptx-ft codegen run [options]
```

#### 全局选项

- `-i, --input <path>` - 输入 OpenAPI 路径/URL
- `-p, --plugin <paths...>` - 追加插件路径
- `--terminals <ids...>` - 指定要生成的终端（如 axios-ts、react-query）
- `--output <dir>` - 输出根目录

#### 命令选项

- `--dry-run` - 只构建生成计划，不实际写入文件
- `--profile` - 输出执行耗时统计
- `--report-json <file>` - 输出结构化执行报告到 JSON 文件
- `--concurrency <auto|N>` - 覆盖并发度设置（auto 或具体数字）
- `--cache <true|false>` - 覆盖增量缓存开关

#### 示例

```bash
# 基本使用
aptx-ft codegen run -i ./openapi.json --terminals axios-ts react-query --output ./generated

# 使用远程 OpenAPI
aptx-ft codegen run -i https://api.example.com/openapi.json --terminals axios-ts --output ./generated

# 仅预览生成计划
aptx-ft codegen run --dry-run

# 输出性能报告
aptx-ft codegen run --profile

# 输出 JSON 报告
aptx-ft codegen run --report-json ./report.json

# 设置并发数为 4
aptx-ft codegen run --concurrency 4

# 禁用缓存
aptx-ft codegen run --cache false
```

### 2.2 `model:gen` - 生成 TypeScript 类型声明

从 OpenAPI schemas 生成 TypeScript 模型声明。

#### 语法

```bash
aptx-ft model gen [options]
```

#### 选项

- `--output <dir>` - 输出目录（必需）
- `--style <declaration|module>` - 输出样式
  - `declaration`：生成 `.d.ts` 声明文件（默认）
  - `module`：生成带 `export interface` 的 `.ts` 模块文件
- `--name <schema>` - 仅生成指定 schema 名称（可重复）

#### 示例

```bash
# 生成声明文件
aptx-ft model gen --output ./generated/models

# 生成模块文件
aptx-ft model gen --output ./generated/models --style module

# 仅生成特定 schema
aptx-ft model gen --output ./generated/models --name Order --name User

# 组合使用
aptx-ft model gen --output ./generated/models --style module --name Order
```

### 2.3 `model:ir` - 导出模型 IR 快照

导出模型中间表示的 JSON 快照。

#### 语法

```bash
aptx-ft model ir [options]
```

#### 选项

- `--output <file>` - 输出 JSON 文件路径（必需）

#### 示例

```bash
aptx-ft model ir --output ./tmp/model-ir.json
```

### 2.4 `model:enum-plan` - 导出枚举增强计划 JSON

导出枚举增强计划，用于后续枚举补丁操作。

#### 语法

```bash
aptx-ft model enum-plan [options]
```

#### 选项

- `--output <file>` - 输出 JSON 文件路径（必需）

#### 示例

```bash
aptx-ft model enum-plan --output ./tmp/enum-plan.json
```

### 2.5 `model:enum-apply` - 应用枚举补丁并生成模型

应用枚举补丁文件并生成增强的模型文件。

#### 语法

```bash
aptx-ft model enum-apply [options]
```

#### 选项

- `--output <dir>` - 输出目录（必需）
- `--patch <file>` - 枚举补丁 JSON 文件路径（必需）
- `--style <declaration|module>` - 输出样式（默认：`declaration`）
- `--conflict-policy <openapi-first|patch-first|provider-first>` - 冲突策略（默认：`patch-first`）
  - `openapi-first`：优先使用 OpenAPI 定义的枚举成员
  - `patch-first`：优先使用补丁中的枚举成员
  - `provider-first`：优先使用数据源提供的枚举成员
- `--name <schema>` - 仅生成指定 schema（可重复）

#### 示例

```bash
# 基本使用
aptx-ft model enum-apply --patch ./tmp/enum-patch.json --output ./generated/models

# 指定样式和冲突策略
aptx-ft model enum-apply --patch ./tmp/enum-patch.json --output ./generated/models --style module --conflict-policy patch-first

# 仅生成特定 schema
aptx-ft model enum-apply --patch ./tmp/enum-patch.json --output ./generated/models --name Order
```

#### 枚举补丁 JSON 格式

枚举补丁文件必须符合以下格式（**注意：使用 `suggested_name` 而非 `name`**）：

```json
{
  "schema_version": "1",
  "patches": [
    {
      "enum_name": "AssignmentStatus",
      "members": [
        { "value": "0", "suggested_name": "Enabled", "comment": "启用" },
        { "value": "1", "suggested_name": "Disabled", "comment": "禁用" },
        { "value": "2", "suggested_name": "Banned", "comment": "封禁" }
      ]
    }
  ]
}
```

**字段说明**：
- `enum_name`：枚举名称，必须与 OpenAPI 中的枚举名称匹配
- `members`：枚举成员数组
  - `value`：枚举值（字符串）
  - `suggested_name`：建议的成员名称（**必填**，用于替换 Value1/Value2 等默认名称）
  - `comment`：成员注释（可选）

#### 枚举优化完整流程

```bash
# 步骤 1：导出枚举计划（查看当前枚举情况）
aptx-ft -i ./openapi.json model enum-plan --output ./tmp/enum-plan.json

# 步骤 2：从 Materal API 获取枚举补丁（如果后端支持）
aptx-ft -i ./openapi.json materal:enum-patch --base-url http://localhost:5000 --output ./tmp/enum-patch.json

# 步骤 3：手动编辑补丁文件，优化 suggested_name（重要！）
# 将 "AssignmentStatusValue0" 改为 "Enabled" 等有意义的名称

# 步骤 4：应用补丁并生成模型
aptx-ft -i ./openapi.json model enum-apply --patch ./tmp/enum-patch.json --output ./generated/models --style module
```

### 2.6 `ir:snapshot` - 导出 IR 快照 JSON

导出完整的中间表示快照，供脚本插件使用。

#### 语法

```bash
aptx-ft ir:snapshot [options]
```

#### 选项

- `--output <file>` - 输出 JSON 文件路径（必需）

#### 示例

```bash
aptx-ft ir:snapshot --output ./tmp/ir.json
```

### 2.7 `terminal:codegen` - 按单 terminal 执行内置生成

为单个内置终端执行代码生成。

#### 语法

```bash
aptx-ft terminal:codegen [options]
```

#### 选项

- `--terminal <id>` - 终端 ID（必需），如 `axios-ts`、`react-query`
- `--output <dir>` - 输出目录（必需）

#### 示例

```bash
aptx-ft terminal:codegen --terminal axios-ts --output ./generated/services/axios-ts
```

### 2.8 `materal:antd-init` - 生成 Ant Design 脚手架

从 OpenAPI 生成 Ant Design 物料脚手架。

#### 语法

```bash
aptx-ft materal:antd-init [options]
```

#### 选项

- `--store <boolean>` - 是否生成字典 store

#### 示例

```bash
aptx-ft materal:antd-init -i ./openapi.json --store true
```

### 2.9 `materal:enum-patch` - 获取 Materal 枚举值并输出补丁

从 Materal API 获取枚举值并输出标准枚举补丁 JSON。

#### 语法

```bash
aptx-ft materal:enum-patch [options]
```

#### 选项

- `--base-url <url>` - Materal API 基础 URL（必需）
- `--output <file>` - 输出补丁 JSON 文件路径（必需）
- `--max-retries <n>` - HTTP 重试次数（默认：3）
- `--timeout-ms <ms>` - HTTP 超时毫秒数（默认：10000）
- `--naming-strategy <auto|none>` - 枚举成员命名策略（默认：`auto`）

#### 示例

```bash
aptx-ft -i ./openapi.json materal:enum-patch --base-url http://localhost:5000 --output ./tmp/enum-patch.json

# 自定义超时和重试
aptx-ft -i ./openapi.json materal:enum-patch --base-url http://localhost:5000 --output ./tmp/enum-patch.json --timeout-ms 30000 --max-retries 5
```

### 2.10 其他实用命令

#### `codegen list-terminals` - 列出终端支持状态

```bash
aptx-ft codegen list-terminals
```

#### `doctor` - 运行健康检查

```bash
aptx-ft doctor
```

输出包括：
- Node 版本
- Binding 可用性
- 已注册命令列表
- Script 插件加载数量

#### `plugin list` - 查看插件提供方

```bash
aptx-ft plugin list
```

#### `input download` - 下载远程 OpenAPI JSON

```bash
aptx-ft input download --url <url> --output <file>
```

示例：
```bash
aptx-ft input download --url http://localhost:5000/swagger/v1/swagger.json --output ./openapi.json
```

## 3. 内置 Terminal 说明

### 3.1 `axios-ts`

**输出特征**：
- 按命名空间（第一个标签）分组生成 TypeScript 类
- 每个类继承自 `BaseService`
- 使用 `tsyringe` 的 `@singleton()` 装饰器
- 类名格式：`{Namespace}Service`（PascalCase）
- 文件名格式：`{Namespace}Service.ts`

**适用场景**：
- 使用 TypeScript 的 Axios 项目
- 需要依赖注入和单例模式的项目
- 需要按服务模块组织的项目

**输出示例**：
```typescript
import { singleton } from "tsyringe";
import { BaseService } from "./BaseService";

@singleton()
export class UserService extends BaseService {
  GetUserList(input: GetUserListInput) {
    return this.get<UserListOutput>("/api/users", { params: { page: input.page, size: input.size } });
  }

  CreateUser(input: CreateUserInput) {
    return this.post<UserOutput>("/api/users", input.body);
  }
}
```

### 3.2 `axios-js`

**输出特征**：
- 单个 `index.js` 文件
- 纯 JavaScript，无类型信息
- 使用 axios 直接发起请求
- 函数名格式：PascalCase 操作名

**适用场景**：
- JavaScript 项目
- 不需要 TypeScript 的项目
- 简单的 API 调用场景

**输出示例**：
```javascript
import axios from "axios";

export function GetUserList(input) {
  return axios.request({
    url: "/api/users",
    method: "get",
    params: { page: input?.page, size: input?.size }
  });
}
```

### 3.3 `uniapp`

**输出特征**：
- 按命名空间分组生成 TypeScript 类
- 类名格式：`{Namespace}Service`（PascalCase）
- 文件名格式：`{Namespace}Service.ts`
- 类继承自 `BaseService`
- 使用 `tsyringe` 的 `@singleton()` 装饰器

**适用场景**：
- UniApp 项目
- 需要适配 UniApp 网络请求 API 的项目
- 跨平台移动应用开发

**输出示例**：
```typescript
import { singleton } from "tsyringe";
import { BaseService } from "./BaseService";

@singleton()
export class UserService extends BaseService {
  GetUserList(input: GetUserListInput) {
    return this.get<UserListOutput>("/api/users", { params: { page: input.page, size: input.size } });
  }
}
```

### 3.4 `functions`

**输出特征**：
- 为每个端点生成两个文件：
  1. 规格文件：`spec/endpoints/{namespace}/{operation}.ts`
  2. 函数文件：`api/functions/{namespace}/{operation}.ts`
- 规格文件导出 `build{Operation}Spec` 函数
- 函数文件导出 `{operation}` 函数

**适用场景**：
- 需要分离 API 规格和实现的架构
- 函数式编程风格
- 需要高度自定义请求处理的场景

**输出示例**：
```typescript
// spec/endpoints/users/getUserList.ts
export function buildGetUserListSpec(input: GetUserListInput) {
  return {
    method: "GET",
    path: "/api/users",
    query: { page: input.page, size: input.size },
  };
}

// api/functions/users/getUserList.ts
export function getUserList(
  input: GetUserListInput,
  options?: PerCallOptions
): Promise<UserListOutput> {
  return getApiClient().execute<UserListOutput>(buildGetUserListSpec(input), options);
}
```

### 3.5 `react-query`

**输出特征**：
- 为支持查询的端点生成 `{operation}.query.ts` 文件
- 为支持变更的端点生成 `{operation}.mutation.ts` 文件
- 导出 `use{Operation}Query` 和 `use{Operation}Mutation` hooks
- 使用 `@aptx/api-query-adapter` 和 `@aptx/react-query`

**适用场景**：
- 使用 React Query 的 React 项目
- 需要数据缓存和状态管理
- 需要自动重新获取和乐观更新的场景

**输出示例**：
```typescript
// api/react-query/users/getUserList.query.ts
import { createQueryDefinition } from "@aptx/api-query-adapter";
import { createReactQueryHooks } from "@aptx/react-query";

export const getUserListQueryDef = createQueryDefinition<GetUserListInput, UserListOutput>({
  keyPrefix: ["users", "getUserList"],
  buildSpec: buildGetUserListSpec,
  execute: (spec, options, queryContext) => getApiClient().execute(spec, options),
});

export const { useAptxQuery: useGetUserListQuery } = createReactQueryHooks(getUserListQueryDef);
```

### 3.6 `vue-query`

**输出特征**：
- 为支持查询的端点生成 `{operation}.query.ts` 文件
- 为支持变更的端点生成 `{operation}.mutation.ts` 文件
- 导出 `use{Operation}Query` 和 `use{Operation}Mutation` composables
- 使用 `@aptx/api-query-adapter` 和 `@aptx/vue-query`

**适用场景**：
- 使用 Vue Query 的 Vue 项目
- 需要组合式 API 的 Vue 3 项目
- 需要数据缓存和状态管理

**输出示例**：
```typescript
// api/vue-query/users/getUserList.query.ts
import { createQueryDefinition } from "@aptx/api-query-adapter";
import { createVueQueryHooks } from "@aptx/vue-query";

export const getUserListQueryDef = createQueryDefinition<GetUserListInput, UserListOutput>({
  keyPrefix: ["users", "getUserList"],
  buildSpec: buildGetUserListSpec,
  execute: (spec, options, queryContext) => getApiClient().execute(spec, options),
});

export const { useAptxQuery: useGetUserListQuery } = createVueQueryHooks(getUserListQueryDef);
```

## 4. Script Plugin 扩展开发指南

### 4.1 插件协议

Script Plugin 必须导出以下字段：

```typescript
{
  apiVersion: string;      // 必须为 "1"
  pluginName: string;      // 插件名称
  pluginVersion: string;   // 插件版本
  commands?: ScriptPluginCommand[];   // 命令列表
  renderers?: ScriptPluginRenderer[]; // 渲染器列表
}
```

### 4.2 命令注册

#### ScriptPluginCommand 类型

```typescript
type ScriptPluginCommand = {
  name: string;              // 命令名称
  summary?: string;          // 简短描述
  description?: string;      // 详细描述
  options?: HelpOptionDescriptor[];  // 选项描述
  examples?: string[];       // 使用示例
  run: (ctx: {
    args: string[];         // 命令参数
    input?: string;         // 输入源
    config: AppConfig;      // 应用配置
    getIrSnapshot: () => Promise<GeneratorInputIR>;  // 获取 IR 快照
  }) => void | Promise<void>;
};
```

#### 命令示例

```javascript
/** @type {import("node:module")} */
module.exports = {
  apiVersion: "1",
  pluginName: "demo-plugin",
  pluginVersion: "0.1.0",
  commands: [
    {
      name: "demo:count",
      summary: "Print endpoint count",
      description: "Counts and displays the number of endpoints in the OpenAPI spec.",
      options: [
        {
          long: "verbose",
          short: "v",
          description: "Enable verbose output",
          required: false,
          multiple: false,
        },
      ],
      examples: [
        "aptx-ft demo:count",
        "aptx-ft demo:count --verbose",
      ],
      run: async ({ args, input, config, getIrSnapshot }) => {
        const ir = await getIrSnapshot();
        console.log(`Endpoint count: ${ir.endpoints.length}`);
        console.log(`Project: ${ir.project.package_name}`);
      },
    },
  ],
};
```

### 4.3 Renderer 注册

#### ScriptPluginRenderer 类型

```typescript
type ScriptPluginRenderer = {
  id: string;              // 渲染器 ID，对应 terminal ID
  render: (ctx: {
    input: string;          // 输入源
    ir: GeneratorInputIR;    // IR 数据
    terminal: TerminalConfig; // 终端配置
    outputRoot: string;      // 输出根目录
    config: AppConfig;       // 应用配置
    writeFile: (filePath: string, content: string) => void;      // 写入单个文件
    writeFiles: (files: Array<{ path: string; content: string }>) => void;  // 批量写入
  }) => void | Promise<void>;
};
```

#### Renderer 示例

```javascript
/** @type {import("node:module")} */
module.exports = {
  apiVersion: "1",
  pluginName: "demo-renderer",
  pluginVersion: "0.1.0",
  renderers: [
    {
      id: "demo-terminal",
      render: async ({ ir, writeFile, writeFiles }) => {
        // 写入单个文件
        writeFile(
          "index.ts",
          `export const endpointCount = ${ir.endpoints.length};\n`
        );

        // 批量写入文件
        writeFiles(
          ir.endpoints.map(endpoint => ({
            path: `${endpoint.operation_name}.ts`,
            content: `// ${endpoint.operation_name}\nexport const path = "${endpoint.path}";\n`,
          }))
        );
      },
    },
  ],
};
```

### 4.4 安全策略说明

Script Plugin 受以下安全策略限制：

1. **超时限制**：执行时间超过 `timeoutMs` 将被终止
2. **文件数量限制**：写入文件数不能超过 `maxWriteFiles`
3. **字节数限制**：写入总字节数不能超过 `maxWriteBytes`
4. **堆内存限制**：堆内存使用不能超过 `maxHeapMb` MB
5. **路径限制**：只能写入终端输出根目录内，不能逃逸

### 4.5 完整示例

```javascript
/** @type {import("node:module")} */
module.exports = {
  apiVersion: "1",
  pluginName: "my-custom-plugin",
  pluginVersion: "1.0.0",

  commands: [
    {
      name: "my-plugin:analyze",
      summary: "Analyze OpenAPI spec",
      run: async ({ getIrSnapshot }) => {
        const ir = await getIrSnapshot();
        const stats = {
          totalEndpoints: ir.endpoints.length,
          namespaces: [...new Set(ir.endpoints.flatMap(e => e.namespace))].length,
          hasQuerySupport: ir.endpoints.filter(e => e.supports_query).length,
          hasMutationSupport: ir.endpoints.filter(e => e.supports_mutation).length,
        };
        console.log(JSON.stringify(stats, null, 2));
      },
    },
  ],

  renderers: [
    {
      id: "my-custom-terminal",
      render: async ({ ir, writeFile, writeFiles, outputRoot }) => {
        // 生成索引文件
        writeFile(
          "index.ts",
          `// Auto-generated by my-custom-plugin\n` +
          `export const ENDPOINT_COUNT = ${ir.endpoints.length};\n`
        );

        // 为每个命名空间生成文件
        const byNamespace = new Map();
        for (const endpoint of ir.endpoints) {
          const ns = endpoint.namespace[0] || "default";
          if (!byNamespace.has(ns)) byNamespace.set(ns, []);
          byNamespace.get(ns).push(endpoint);
        }

        const files = [];
        for (const [ns, endpoints] of byNamespace) {
          files.push({
            path: `${ns}.ts`,
            content: `// ${ns} namespace\n` +
              endpoints.map(e => `export const ${e.operation_name} = "${e.path}";`).join("\n") +
              "\n",
          });
        }
        writeFiles(files);
      },
    },
  ],
};
```

## 5. Native Plugin 扩展开发指南

### 5.1 插件结构

Native Plugin 使用 Rust 编写，编译为动态库（`.dll`、`.so`、`.dylib`）。

### 5.2 init_plugin 导出函数

插件必须导出 `init_plugin` 函数：

```rust
use aptx_frontend_tk_binding_plugin::command::{CommandDescriptor, CommandRegistry, OptionDescriptor};

#[no_mangle]
pub extern "C" fn init_plugin(command: &CommandRegistry) {
    // 注册命令
}
```

### 5.3 CommandDescriptor 使用

```rust
command.register_command_with_descriptor(
    CommandDescriptor {
        name: "my:command".to_string(),
        summary: "My custom command".to_string(),
        description: Some("Detailed description".to_string()),
        options: vec![
            OptionDescriptor {
                long: "option".to_string(),
                value_name: Some("value".to_string()),
                required: true,
                description: "Option description".to_string(),
                ..Default::default()
            },
        ],
        examples: vec![
            "aptx-ft my:command --option value".to_string(),
        ],
        plugin_name: Some("my-plugin".to_string()),
        plugin_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        ..Default::default()
    },
    Box::new(|args, open_api| {
        // 命令执行逻辑
        Ok(())
    }),
);
```

### 5.4 完整示例

```rust
use aptx_frontend_tk_binding_plugin::command::{
    CommandDescriptor, CommandRegistry, OptionDescriptor,
};

#[no_mangle]
pub extern "C" fn init_plugin(command: &CommandRegistry) {
    command.register_command_with_descriptor(
        CommandDescriptor {
            name: "my-plugin:generate".to_string(),
            summary: "Generate custom artifacts".to_string(),
            description: Some("Generates custom artifacts from OpenAPI".to_string()),
            options: vec![
                OptionDescriptor {
                    long: "output".to_string(),
                    value_name: Some("dir".to_string()),
                    required: true,
                    description: "Output directory".to_string(),
                    ..Default::default()
                },
            ],
            examples: vec![
                "aptx-ft my-plugin:generate --output ./custom-output".to_string(),
            ],
            plugin_name: Some("my-native-plugin".to_string()),
            plugin_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            ..Default::default()
        },
        Box::new(|args, open_api| {
            // 解析参数
            let output = args.iter()
                .position(|a| a == "--output")
                .and_then(|i| args.get(i + 1))
                .ok_or("Missing --output argument")?;

            // 执行生成逻辑
            // ...

            Ok(())
        }),
    );
}
```

### 5.5 编译和分发

1. 在 `Cargo.toml` 中配置：

```toml
[lib]
crate-type = ["cdylib"]
```

2. 编译：

```bash
cargo build --release
```

3. 使用 `-p` 参数引用插件：

```bash
aptx-ft -i ./openapi.json -p ./target/release/libmy_native_plugin.dll codegen run --terminals axios-ts --output ./generated
```

## 6. IR 结构说明

### 6.1 GeneratorInputIR

```typescript
type GeneratorInputIR = {
  project: GeneratorProjectContextIR;
  endpoints: GeneratorEndpointIR[];
};
```

### 6.2 GeneratorProjectContextIR

```typescript
type GeneratorProjectContextIR = {
  package_name: string;
  api_base_path?: string | null;
  terminals: string[];
  retry_ownership?: string | null;
};
```

### 6.3 GeneratorEndpointIR

```typescript
type GeneratorEndpointIR = {
  namespace: string[];
  operation_name: string;
  method: string;
  path: string;
  input_type_name: string;
  output_type_name: string;
  request_body_field?: string | null;
  query_fields: string[];
  path_fields: string[];
  has_request_options: boolean;
  supports_query: boolean;
  supports_mutation: boolean;
  deprecated: boolean;
};
```

## 7. 故障排查

### 7.1 常见错误

#### `input is required`

缺少输入源配置。解决方法：
- 使用 `-i` 选项指定输入源

#### `terminals is required`

未指定要生成的终端。解决方法：
- 使用 `--terminals` 选项指定至少一个终端

#### `Terminal ... not supported`

终端不被支持。解决方法：
- 检查终端名称是否为内置终端
- 确认已注册对应的 Script Plugin renderer

#### Script 超时或写入超限

解决方法：
- 拆分生成任务

### 7.2 调试技巧

1. 使用 `--dry-run` 预览生成计划
2. 使用 `--profile` 查看性能瓶颈
3. 使用 `aptx-ft doctor` 检查环境状态
4. 检查缓存文件 `<outputRoot>/.aptx-cache/run-cache.json`

## 8. 附录

### 8.1 内置终端列表

| ID | 状态 | 说明 |
|-----|------|------|
| axios-ts | available | TypeScript Axios 服务类 |
| axios-js | available | JavaScript Axios 函数 |
| uniapp | available | UniApp 服务类 |
| functions | available | 函数式规格和实现 |
| react-query | available | React Query Hooks |
| vue-query | available | Vue Query Composables |

### 8.2 相关文档

- 架构文档：`docs/final-codegen-architecture.md`
- 使用说明：`docs/codegen-usage.md`
