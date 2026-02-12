# Frontend Toolkit (aptx-ft) 架构文档

## 项目概述

Frontend Toolkit 是一个基于 Rust 后端 + TypeScript CLI 的代码生成工具，用于根据 OpenAPI/Swagger 3.x 规范生成前端 API 客户端、TypeScript 类型定义和各种框架相关的代码。

**技术栈**：
- Rust (核心代码生成引擎)
- TypeScript + Node.js (CLI 界面)
- N-API (Rust ↔ Node.js 跨语言绑定)
- pnpm (包管理器)

---

## 整体架构

```mermaid
graph TB
    subgraph UserLayer["用户层"]
        CLI["命令行: aptx-ft"]
    end

    subgraph TSCLILayer["TypeScript CLI 层"]
        CLI_ENTRY["frontend-tk-cli 主入口"]
        CONFIG["配置加载 c12"]
        DOWNLOADER["文件下载器 get-input.ts"]
        PLUGIN_MGR["插件管理器"]
        CONCURRENCY["并发控制 缓存管理"]
    end

    subgraph NAPILayer["N-API 绑定层"]
        BINDING["node_binding Rust to Node.js"]
        LOADER["插件加载器 libloading"]
    end

    subgraph RustCoreLayer["Rust 核心层"]
        PLUGIN["node_binding_plugin 命令注册表"]
        GEN["swagger_gen 代码生成引擎"]
        PARSER["解析器 OpenAPI to IR"]
        RENDERER["渲染器 API 客户端生成"]
        TYPER["类型生成器 TS 声明生成"]
        TK["swagger_tk OpenAPI 模型"]
        MACRO["swagger_macro 派生宏"]
        MATERAL["frontend_plugin_materal 示例插件"]
    end

    CLI --> CLI_ENTRY
    CLI_ENTRY --> CONFIG
    CLI_ENTRY --> DOWNLOADER
    CLI_ENTRY --> PLUGIN_MGR
    CLI_ENTRY --> CONCURRENCY
    CLI_ENTRY -->|"N-API 调用"| BINDING
    BINDING --> LOADER
    BINDING --> PLUGIN
    BINDING --> GEN
    PLUGIN --> GEN
    GEN --> PARSER
    GEN --> RENDERER
    GEN --> TYPER
    PARSER --> TK
    RENDERER --> TK
    TYPER --> TK
    TK --> MACRO
    LOADER --> MATERAL

    style CLI fill:#e1f5ff
    style BINDING fill:#fff4e1
    style GEN fill:#e8f5e9
    style TK fill:#fce4ec
```

---

## Rust Crate 依赖关系

```mermaid
graph LR
    subgraph Layer1["第 1 层: 基础工具"]
        MACRO["swagger_macro 派生宏"]
    end

    subgraph Layer2["第 2 层: 模型与工具"]
        TK["swagger_tk OpenAPI 模型"]
    end

    subgraph Layer3["第 3 层: 代码生成"]
        GEN["swagger_gen 代码生成引擎"]
    end

    subgraph Layer4["第 4 层: 插件抽象"]
        PLUGIN["node_binding_plugin 插件系统"]
    end

    subgraph Layer5["第 5 层: N-API 绑定"]
        BINDING["node_binding Rust to Node.js"]
    end

    subgraph Layer6["第 6 层: 示例插件"]
        MATERAL["frontend_plugin_materal 示例插件"]
    end

    TK --> MACRO
    GEN --> TK
    PLUGIN --> GEN
    PLUGIN --> TK
    BINDING --> PLUGIN
    BINDING --> GEN
    MATERAL --> PLUGIN
    MATERAL --> GEN
    MATERAL --> TK

    style MACRO fill:#90caf9
    style TK fill:#ce93d8
    style GEN fill:#a5d6a7
    style PLUGIN fill:#ffcc80
    style BINDING fill:#ef9a9a
    style MATERAL fill:#bcaaa4
```

### 详细依赖说明

| Crate | 职责 | 外部依赖 | 被依赖 |
|--------|------|----------|---------|
| **swagger_macro** | Rust 过程宏，提供派生能力 | `quote`, `syn` | swagger_tk |
| **swagger_tk** | OpenAPI 3.x 完整模型定义 | `swagger_macro`, `serde`, `serde_json` | swagger_gen, node_binding_plugin, frontend_plugin_materal |
| **swagger_gen** | 代码生成核心引擎 | `swagger_tk`, `Inflector`, `dprint-*`, `regex` | node_binding_plugin, node_binding, frontend_plugin_materal |
| **node_binding_plugin** | 插件系统接口定义 | `swagger_gen`, `swagger_tk` | node_binding, frontend_plugin_materal |
| **node_binding** | N-API 绑定层 | `napi`, `napi-derive`, `libloading`, `node_binding_plugin`, `swagger_gen`, `swagger_tk`, `frontend_plugin_materal` | frontend-tk-cli |
| **frontend_plugin_materal** | 示例插件 | `node_binding_plugin`, `swagger_gen`, `swagger_tk` | - |

---

## TypeScript Package 依赖关系

```mermaid
graph LR
    subgraph Layer1["第 1 层: 类型定义"]
        TYPES["frontend-tk-types 配置类型"]
    end

    subgraph Layer2["第 2 层: CLI 应用"]
        CLI["frontend-tk-cli 命令行工具"]
    end

    subgraph BindingLayer["Native 绑定"]
        BINDING["@aptx/frontend-tk-binding N-API 模块"]
        RUST(("Rust node_binding"))
    end

    CLI --> BINDING
    CLI --> TYPES
    BINDING -->|"编译为"| RUST

    style TYPES fill:#b39ddb
    style CLI fill:#80cbc4
    style BINDING fill:#81d4fa
```

### Package 详细说明

| Package | 职责 | 外部依赖 |
|---------|------|----------|
| **frontend-tk-types** | 配置文件类型定义<br/>`defineConfig()` 辅助函数 | - |
| **frontend-tk-cli** | 用户 CLI 界面<br/>命令解析、配置加载、文件下载<br/>插件管理、并发控制 | `@aptx/frontend-tk-binding`, `c12`, `chalk`, `commander` |
| **@aptx/frontend-tk-binding** | N-API 绑定模块<br/>编译为跨平台 .node 二进制 | `napi`, `napi-derive`, `libloading` |

---

## 模块结构详解

### swagger_tk - OpenAPI 模型层

```
swagger_tk/src/
├── model/                          # OpenAPI 3.x 完整类型定义
│   ├── open_api_object.rs          # 顶层 OpenAPI 对象
│   ├── operation_object.rs         # 操作对象
│   ├── path_item_object.rs         # 路径项对象
│   ├── response_object.rs          # 响应对象
│   ├── request_body_object.rs      # 请求体对象
│   ├── parameter_object.rs        # 参数对象
│   ├── media_type_object.rs        # 媒体类型
│   ├── schema/                    # Schema 类型系统
│   │   ├── mod.rs
│   │   ├── schema_enum.rs       # Schema 枚举类型
│   │   ├── schema_object.rs     # 对象类型
│   │   ├── schema_array.rs      # 数组类型
│   │   ├── schema_string.rs     # 字符串类型
│   │   ├── schema_integer.rs    # 整数类型
│   │   ├── schema_number.rs     # 数字类型
│   │   └── schema_bool.rs      # 布尔类型
│   └── ... (300+ 个文件)
├── extension/                      # 模型扩展方法
│   ├── schema_enum.rs            # Schema → TS 类型转换
│   ├── media_type_object.rs      # 媒体类型扩展
│   └── response_value.rs       # 响应值扩展
└── getter/                        # 查询工具
    ├── get_schema_by_name.rs    # 通过名称获取 Schema
    ├── get_schema_name_from_ref.rs  # 从 $ref 解析名称
    ├── get_tags.rs              # 获取 tags
    └── get_controller_description.rs  # 获取控制器描述
```

**核心功能**：
- 解析 OpenAPI 3.x JSON/YAML
- 提供类型安全的 Rust 模型
- Schema 到 TypeScript 类型转换 (`get_ts_type()`)
- 查询辅助方法

---

### swagger_gen - 代码生成引擎

```
swagger_gen/src/
├── core/                          # API 上下文构建
│   ├── js_helper.rs              # API 上下文主结构
│   └── js_api_context_helper.rs  # 上下文辅助方法
├── gen_api/                       # API 客户端生成器
│   ├── mod.rs
│   ├── axios_ts.rs               # Axios + TypeScript
│   ├── axios_js.rs               # Axios + JavaScript
│   └── uni_app.rs               # UniApp 框架
├── gen_declaration/                # TypeScript 类型生成
│   ├── declaration.rs            # 类型声明生成
│   └── index.ts                # 导出生成
├── pipeline/                       # 代码生成流水线
│   ├── parser.rs                 # OpenAPI → IR 转换
│   └── model.rs                 # 中间表示定义
├── utils/                         # 工具函数
│   ├── format_ts_code.rs         # 代码格式化 (dprint)
│   ├── schema_extension.rs       # Schema → TS 类型扩展
│   └── reference_object_extension.rs  # 引用对象扩展
└── lib.rs                        # 库入口
```

**核心功能**：
- 解析 OpenAPI 到中间表示 (IR)
- 生成多种框架的 API 客户端
- 生成 TypeScript 类型声明
- 代码格式化与美化

---

### node_binding - N-API 绑定层

```
node_binding/src/
├── lib.rs                         # N-API 入口，暴露函数
│   ├── run_cli()                 # 执行 CLI 命令
│   └── get_help_tree()          # 获取帮助树
├── bootstrap.rs                   # 命令工厂初始化
│   └── init_command_factory()    # 加载插件并初始化
├── built_in/                      # 内置命令
│   ├── mod.rs                   # 命令注册入口
│   ├── ir.rs                   # ir:snapshot 命令
│   └── terminal_codegen.rs      # terminal:codegen 命令
└── package.json                   # N-API 配置
    ├── targets                  # 支持的平台列表
    └── binaryName               # 二进制名称
```

**暴露的 N-API 函数**：

```typescript
// packages/frontend-tk-cli/src/index.ts
import { runCli, getHelpTree } from "@aptx/frontend-tk-binding";

// 执行命令
runCli({
  input: string,           // OpenAPI 文件路径
  command: string,        // 命令名称
  plugin?: string[],      // 插件路径列表
  options: string[]       // 命令选项
})

// 获取帮助树
getHelpTree({ plugin?: string[] })
  → HelpCommandDescriptor[]
```

---

### frontend-tk-cli - CLI 应用层

```
frontend-tk-cli/src/
├── index.ts                       # 主入口，命令路由
├── config.ts                      # 配置加载 (c12)
├── command/
│   ├── input/
│   │   └── get-input.ts        # OpenAPI 文件下载/读取
│   └── gen/                    # 代码生成流程
└── utils.ts                       # 工具函数

bin/aptx.js                       # 可执行入口
dist/                             # 编译输出
types/                            # 类型声明
```

**支持的命令**：

| 命令 | 功能 |
|------|------|
| `codegen run` | 执行代码生成 |
| `codegen list-terminals` | 列出内置终端 |
| `doctor` | 检查环境状态 |
| `plugin list` | 列出加载的插件 |

---

## 完整调用链流程

### 主流程：代码生成

```mermaid
sequenceDiagram
    participant User as 用户
    participant CLI as frontend-tk-cli
    participant Config as 配置加载器
    participant Downloader as 文件下载器
    participant NAPI as N-API 绑定
    participant Parser as OpenAPI 解析器
    participant IR as 中间表示(IR)
    participant Gen as 代码生成器
    participant Render as 渲染器
    participant Output as 文件系统

    User->>CLI: aptx-ft codegen run -c config.ts
    CLI->>Config: 加载配置文件
    Config-->>CLI: APTXFtConfig
    CLI->>Downloader: 下载 OpenAPI 文件 (如果 input 是 URL)
    Downloader->>Downloader: HTTP GET / 下载到临时目录
    Downloader-->>CLI: 本地文件路径
    CLI->>CLI: 解析插件配置
    CLI->>NAPI: runCli({input, command, options, plugin})
    NAPI->>Parser: 读取 OpenAPI JSON
    Parser->>Parser: 解析为 OpenAPIObject
    Parser->>IR: 构建 GeneratorInput
    IR->>IR: 遍历 paths/operations
    IR->>IR: 构建 EndpointItem 列表
    IR-->>Gen: GeneratorInput (IR)
    Gen->>Gen: 执行 pipeline

    par 并发生成多个终端
        Gen->>Render: 生成 axios-ts
        Render->>Output: 写入文件
        Gen->>Render: 生成 react-query
        Render->>Output: 写入文件
        Gen->>Render: 生成 uniapp
        Render->>Output: 写入文件
    end

    Gen-->>NAPI: 生成完成
    NAPI-->>CLI: 返回结果
    CLI->>User: 输出报告
```

---

### 子流程：OpenAPI 解析

```mermaid
flowchart TD
    A["OpenAPI JSON/YAML 文件"] --> B["serde_json::from_str"]
    B --> C{"解析成功?"}
    C -->|"否"| D["抛出错误"]
    C -->|"是"| E["OpenAPIObject"]

    E --> F["提取 info 字段"]
    E --> G["提取 paths 字段"]
    E --> H["提取 components/schemas"]

    G --> I["遍历每个 PathItem"]
    I --> J["遍历每个 Operation: get post put delete..."]
    J --> K["构建 ApiContext"]

    K --> L["提取 operation_id 或生成函数名"]
    K --> M["提取参数"]
    K --> N["提取请求体"]
    K --> O["提取响应"]

    L --> P["生成 GeneratorInput"]
    M --> P
    N --> P
    O --> P

    style A fill:#e3f2fd
    style E fill:#f3e5f5
    style P fill:#e8f5e9
```

---

### 子流程：代码生成流水线

```mermaid
flowchart LR
    IR["GeneratorInput IR"]
    PARSER["Parser"]
    TRANSFORM["Transformer"]
    RENDERER["Renderer"]
    LAYOUT["Layout"]
    WRITER["Writer"]
    FILES["生成文件"]

    IR --> PARSER
    PARSER --> TRANSFORM
    TRANSFORM --> RENDERER
    RENDERER --> LAYOUT
    LAYOUT --> WRITER
    WRITER --> FILES

    style IR fill:#fff9c4
    style PARSER fill:#ffcc80
    style TRANSFORM fill:#ffe0b2
    style RENDERER fill:#c8e6c9
    style LAYOUT fill:#a5d6a7
    style FILES fill:#81c784
```

---

### 子流程：插件系统

```mermaid
sequenceDiagram
    participant CLI as CLI
    participant Factory as CommandFactory
    participant Loader as PluginLoader
    participant Registry as CommandRegistry
    participant Plugin as Native Plugin
    participant Script as Script Plugin

    CLI->>Factory: init_command_factory(pluginPaths)
    Factory->>Loader: 遍历插件路径

    alt 原生插件 (.dll/.so)
        Loader->>Plugin: libloading::Library::new(path)
        Loader->>Plugin: 获取 init_plugin 符号
        Plugin->>Registry: 注册命令
        Loader->>Factory: 保存 Library 句柄
    else 脚本插件 (.js)
        Loader->>Script: require(path)
        Script-->>Loader: ScriptPluginModule
        Loader->>Registry: 注册命令/渲染器
    end

    Factory-->>CLI: CommandFactory
    CLI->>Registry: execute_command(name, args, openAPI)
    Registry->>Registry: 查找命令
    Registry->>Registry: 执行回调函数
    Registry-->>CLI: 执行结果
```

---

### 子流程：Schema 到 TypeScript 类型转换

```mermaid
flowchart TD
    A["OpenAPI Schema"] --> B{"Schema 类型"}

    B -->|"String"| S["string"]
    B -->|"Integer"| I["number"]
    B -->|"Number"| N["number"]
    B -->|"Boolean"| BOL["boolean"]
    B -->|"Object"| OBJ["object"]
    B -->|"Array"| ARR["Array<T>"]
    B -->|"Reference"| REF["自定义类型"]

    I --> C["检查 nullable"]
    N --> C
    S --> C
    BOL --> C
    OBJ --> C
    ARR --> C

    C -->|"nullable=true"| D["类型 | null"]
    C -->|"nullable=false"| E["类型"]

    ARR --> F["递归处理 items"]
    F --> E

    REF --> G["查找 components/schemas"]
    G --> E

    style A fill:#f3e5f5
    style B fill:#e1bee7
    style E fill:#c8e6c9
```

**实现代码** (`swagger_gen/src/utils/schema_extension.rs`):

```rust
fn get_ts_type(&self) -> String {
    match self {
        SchemaEnum::Ref(schema) => schema.get_type_name(),
        SchemaEnum::Object(_) => "object".to_string(),
        SchemaEnum::String(_) => "string".to_string(),
        SchemaEnum::Integer(_) => "number".to_string(),
        SchemaEnum::Number(_) => "number".to_string(),
        SchemaEnum::Boolean(_) => "boolean".to_string(),
        SchemaEnum::Array(schema) => {
            let child_type = schema.items.as_ref().get_ts_type();
            format!("Array<{}>", child_type)
        }
    }
}
```

---

## 数据结构

### GeneratorInput (中间表示)

```mermaid
classDiagram
    class GeneratorInput {
        +ProjectContext project
        +GeneratorEndpointIR[] endpoints
    }

    class ProjectContext {
        +string package_name
        +string? api_base_path
        +string[] terminals
        +string? retry_ownership
    }

    class GeneratorEndpointIR {
        +string[] namespace
        +string operation_name
        +string method
        +string path
        +string input_type_name
        +string output_type_name
        +string? request_body_field
        +string[] query_fields
        +string[] path_fields
        +boolean has_request_options
        +boolean supports_query
        +boolean supports_mutation
        +boolean deprecated
    }

    GeneratorInput "1" --> "1" ProjectContext
    GeneratorInput "1" --> "*" GeneratorEndpointIR
```

---

### CommandRegistry (命令注册表)

```mermaid
classDiagram
    class CommandRegistry {
        -HashMap~string,RegisteredCommand~ command_map
        +register_command_with_descriptor(descriptor, callback)
        +list_descriptors() CommandDescriptor[]
        +execute_command(name, args, openAPI) Result
    }

    class RegisteredCommand {
        +CommandDescriptor descriptor
        +CommandFn callback
    }

    class CommandDescriptor {
        +string name
        +string summary
        +string? description
        +string[] aliases
        +OptionDescriptor[] options
        +string[] examples
        +string? plugin_name
        +string? plugin_version
    }

    class CommandContext {
        +string[] args
        +OpenAPIObject open_api
    }

    CommandRegistry "1" --> "*" RegisteredCommand
    RegisteredCommand "1" --> "1" CommandDescriptor
    CommandFn --> CommandContext
```

---

## 配置流程

```mermaid
flowchart TD
    A["用户执行命令"] --> B{"指定 -c 配置文件?"}
    B -->|"是"| C["加载指定配置文件"]
    B -->|"否"| D["查找默认配置文件 aptx-ft.config.ts"]

    C --> E["c12 库解析配置"]
    D --> E

    E --> F["获取 APTXFtConfig"]
    F --> G{"有 input 配置?"}
    G -->|"否"| H["从命令行 -i 参数获取"]
    G -->|"是"| I["使用配置中的 input"]

    H --> J{"input 是 URL?"}
    I --> J

    J -->|"是"| K["下载 OpenAPI 文件"]
    J -->|"否"| L["使用本地文件路径"]

    K --> M["保存到临时目录"]
    L --> M
    M --> N["返回绝对路径"]

    style A fill:#e3f2fd
    style E fill:#fff9c4
    style M fill:#c8e6c9
```

---

## 并发与缓存机制

### 并发执行

```mermaid
flowchart LR
    A["终端列表"] --> B["解析终端配置"]
    B --> C{"终端类型"}

    C -->|"内置终端"| D["Native 任务队列"]
    C -->|"脚本终端"| E["Script 任务队列"]

    D --> F["并发执行器"]
    E --> F

    F --> G["并发数限制 auto = CPU 核心数"]
    G --> H["任务执行池"]

    H --> I["收集执行结果"]
    I --> J["合并报告"]

    style A fill:#e3f2fd
    style F fill:#fff9c4
    style H fill:#c8e6c9
    style J fill:#81c784
```

### 缓存机制

```mermaid
flowchart TD
    A["代码生成请求"] --> B["计算缓存 Key"]
    B --> C["Cache Key = SHA256 inputHash + configHash"]

    C --> D{"缓存目录存在?"}
    D -->|"否"| E["执行完整生成"]
    D -->|"是"| F{"缓存命中?"}

    F -->|"是"| G["跳过生成 使用缓存"]
    F -->|"否"| E

    E --> H["生成文件"]
    H --> I["写入 .aptx-cache/run-cache.json"]
    I --> J["保存 Key 和报告"]

    G --> K["从缓存读取报告"]
    K --> L["返回缓存结果"]

    J --> L

    style A fill:#e3f2fd
    style C fill:#fff9c4
    style G fill:#c8e6c9
    style I fill:#a5d6a7
```

---

## 插件开发指南

### 原生插件开发

```mermaid
flowchart TD
    A["创建 Rust 项目"] --> B["添加依赖 node_binding_plugin"]
    B --> C["实现 init_plugin 函数"]
    C --> D["注册自定义命令"]
    D --> E["编译为 cdylib"]
    E --> F["生成 .dll / .so"]
    F --> G["在 CLI 中指定插件路径 -p path/to/plugin.dll"]

    style A fill:#e3f2fd
    style C fill:#fff9c4
    style F fill:#c8e6c9
    style G fill:#81c784
```

**示例代码**：

```rust
use aptx_frontend_tk_binding_plugin::command::{CommandDescriptor, CommandRegistry, OptionDescriptor};

#[no_mangle]
pub extern "C" fn init_plugin(registry: &CommandRegistry) {
    registry.register_command_with_descriptor(
        CommandDescriptor {
            name: "my-command".to_string(),
            summary: "我的自定义命令".to_string(),
            description: Some("这是一个示例命令".to_string()),
            aliases: vec![],
            options: vec![],
            examples: vec![],
            plugin_name: Some("my-plugin".to_string()),
            plugin_version: Some("1.0.0".to_string()),
        },
        Box::new(|args, open_api| {
            // 命令执行逻辑
            println!("执行我的命令: {:?}", args);
        }),
    );
}
```

### 脚本插件开发

```mermaid
flowchart TD
    A["创建 JS 文件"] --> B["导出插件模块"]
    B --> C["定义 apiVersion, pluginName"]
    C --> D["注册 commands 或 renderers"]
    D --> E["实现命令或渲染器函数"]
    E --> F["在 CLI 中指定插件路径 -p path/to/plugin.js"]

    style A fill:#e3f2fd
    style D fill:#fff9c4
    style E fill:#c8e6c9
```

**示例代码**：

```typescript
module.exports = {
  apiVersion: "1",
  pluginName: "my-script-plugin",
  pluginVersion: "1.0.0",

  commands: [{
    name: "hello",
    summary: "打招呼",
    description: "向用户打招呼",
    options: [{
      long: "name",
      short: "n",
      value_name: "name",
      required: false,
      description: "用户名"
    }],
    examples: ["aptx-ft hello --name World"],
    run: ({ args, input, config, getIrSnapshot }) => {
      const name = args.includes("--name")
        ? args[args.indexOf("--name") + 1]
        : "User";
      console.log(`Hello, ${name}!`);
    }
  }],

  renderers: [{
    id: "custom-terminal",
    render: ({ input, ir, terminal, outputRoot, config, writeFile, writeFiles }) => {
      // 自定义渲染逻辑
      writeFile("custom.js", "// Custom code");
    }
  }]
};
```

---

## 支持的终端

| 终端 ID | 描述 | 生成代码类型 |
|---------|------|------------|
| **axios-ts** | Axios TypeScript 客户端 | `return this.get<T>(url, config)` |
| **axios-js** | Axios JavaScript 客户端 | `return axios.request({...})` |
| **uniapp** | UniApp 框架支持 | `return uni.request({...})` |
| **functions** | 纯函数导出 | `export function apiName(...)` |
| **react-query** | React Query Hooks | `const useApiName = (...)` |
| **vue-query** | Vue Query Composables | `const useApiName = (...)` |

---

## 构建与开发

### Rust 部分

```bash
# 构建 Rust workspace
cargo build --release

# 运行测试
cargo test

# 运行基准测试
cargo bench

# 构建 N-API 绑定
cd crates/node_binding
cargo build --release
napi build --platform --release
```

### TypeScript 部分

```bash
# 安装依赖
pnpm install

# 构建 TypeScript
cd packages/frontend-tk-cli
pnpm run build

# 构建 packages
pnpm -r run build
```

---

## 故障排查

### 常见问题

1. **N-API 模块加载失败**
   - 检查 Node.js 版本 (>= 10)
   - 重新编译 `@aptx/frontend-tk-binding`
   - 确认平台匹配

2. **OpenAPI 解析失败**
   - 验证 OpenAPI 格式
   - 检查 JSON 语法
   - 使用在线工具验证

3. **插件加载失败**
   - 检查插件路径是否正确
   - 原生插件：确认编译为 cdylib
   - 脚本插件：检查 CommonJS 导出格式

---

## 许可证

ISC
