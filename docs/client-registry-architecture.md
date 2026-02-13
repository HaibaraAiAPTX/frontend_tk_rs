# Client/Registry 架构建议文档

**日期**: 2026-02-13
**状态**: 讨论中
**适用范围**: Frontend Toolkit 代码生成器

---

## 1. 当前问题分析

### 1.1 现状

通过分析生成的代码，发现以下特征：

**Functions Terminal** (`test-generate/services/functions/api/functions/`):
```typescript
import type { PerCallOptions } from "../../../client/client";
import { getApiClient } from "../../../client/registry";
```

**React Query Terminal** (`test-generate/services/react-query/api/react-query/`):
```typescript
import { getApiClient } from "../../../client/registry";
```

### 1.2 核心问题

1. **client 和 registry 不存在**: 生成的代码引用了 `../../../client/registry` 和 `../../../client/client`，但这些文件并未被生成
2. **跨终端一致性需求**: 所有终端 (functions, react-query, vue-query, uniapp) 都依赖相同的基础 client 抽象
3. **用户实现负担**: 用户需要自己创建这些文件，但缺乏明确的规范和示例
4. **框架适配问题**: 不同框架可能需要不同的 client 实现 (axios, uni.request, fetch 等)

---

## 2. 架构方案对比

### 方案 A: 独立 npm 包 (推荐)

**描述**: 将 client 和 registry 封装为独立的 npm 包

```
@aptx/api-client/
├── src/
│   ├── client.ts         # ApiClient 接口定义
│   ├── registry.ts       # 默认注册表实现
│   ├── adapters/         # 各种适配器
│   │   ├── axios.ts     # Axios 适配器
│   │   ├── uniapp.ts    # UniApp 适配器
│   │   └── fetch.ts     # Fetch 适配器
│   └── index.ts
├── package.json
└── README.md
```

**优点**:
- 集中维护，统一更新
- 用户只需安装包，无需手动创建文件
- 提供开箱即用的适配器
- 类型定义稳定，便于依赖管理
- 支持 monorepo 场景（单个包被多个项目引用）

**缺点**:
- 需要额外维护一个 npm 包
- 用户自定义需求可能受限
- 版本耦合问题

**实施步骤**:
1. 创建 `@aptx/api-client` 包
2. 定义 `ApiClient` 接口和 `PerCallOptions` 类型
3. 实现常用适配器 (axios, fetch, uniapp)
4. 生成器输出代码引用该包
5. 提供文档说明如何注册自定义 client

---

### 方案 B: 生成器自动生成

**描述**: 生成器自动创建 client 和 registry 文件

```
generated/
├── client/
│   ├── client.ts         # 自动生成的接口定义
│   └── registry.ts      # 自动生成的注册表
└── api/
    └── functions/       # API 函数
```

**优点**:
- 零配置，开箱即用
- 与生成代码版本同步
- 无需额外依赖

**缺点**:
- 每次生成可能覆盖用户自定义
- 用户修改困难
- 代码重复（每个项目都生成相同文件）
- 难以支持 monorepo 共享

**实施步骤**:
1. 在 `renderer.rs` 中添加 client 文件生成逻辑
2. 为每个终端生成对应的 client 文件
3. 提供配置选项控制是否生成

---

### 方案 C: 混合模式（灵活推荐）

**描述**: 提供默认实现 + 用户可选覆盖

```
# 生成器提供模板文件
templates/
└── client/
    ├── client.template.ts
    ├── registry.template.ts
    └── adapters/

# 生成时检测
if (exists(client/registry.ts)) {
  # 使用现有文件
} else {
  # 询问用户是否生成默认实现
}
```

**优点**:
- 平衡灵活性和易用性
- 用户可以选择生成或自己实现
- 支持 monorepo 场景（在 shared 包中创建一次）

**缺点**:
- 需要检测逻辑和交互提示
- 初次使用时需要用户决策

---

## 3. 不同框架的需求分析

| 框架/Terminal | Client 需求 | 推荐适配器 |
|----------------|-------------|-----------|
| **axios-ts** | Axios 实例，拦截器支持 | `@aptx/api-client/adapters/axios` |
| **axios-js** | 同上，无类型 | `@aptx/api-client/adapters/axios` |
| **uniapp** | uni.request API | `@aptx/api-client/adapters/uniapp` |
| **functions** | 任意 HTTP 客户端 | 用户选择 |
| **react-query** | 任意 HTTP 客户端 + abort signal | 用户选择 |
| **vue-query** | 任意 HTTP 客户端 + abort signal | 用户选择 |

### 关键接口设计

```typescript
// client/client.ts
export interface PerCallOptions {
  signal?: AbortSignal;
  headers?: Record<string, string>;
  timeout?: number;
  // ... 其他请求选项
}

export interface ApiClient {
  execute<T>(spec: ApiSpec, options?: PerCallOptions): Promise<T>;
}

export interface ApiSpec {
  method: string;
  path: string;
  query?: Record<string, any>;
  body?: any;
  headers?: Record<string, string>;
}
```

```typescript
// client/registry.ts
import type { ApiClient } from "./client";

let _client: ApiClient | null = null;

export function registerApiClient(client: ApiClient) {
  _client = client;
}

export function getApiClient(): ApiClient {
  if (!_client) {
    throw new Error("ApiClient not registered. Call registerApiClient() first.");
  }
  return _client;
}
```

---

## 4. 推荐实施路径

### 阶段 1: 创建独立 npm 包（方案 A）

```bash
# 创建新包
mkdir packages/api-client
cd packages/api-client
pnpm init
```

**package.json**:
```json
{
  "name": "@aptx/api-client",
  "version": "0.1.0",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "exports": {
    ".": "./dist/index.js",
    "./adapters/axios": "./dist/adapters/axios.js",
    "./adapters/uniapp": "./dist/adapters/uniapp.js",
    "./adapters/fetch": "./dist/adapters/fetch.js"
  },
  "peerDependencies": {
    "axios": "^1.0.0",
    "@tanstack/react-query": "^5.0.0"
  },
  "peerDependenciesMeta": {
    "axios": { "optional": true },
    "@tanstack/react-query": { "optional": true }
  }
}
```

### 阶段 2: 更新生成器引用

修改 `renderer.rs` 中的模板字符串：

```rust
// 生成 functions terminal
format!(
    "import type {{ PerCallOptions }} from \"@aptx/api-client\";
import {{ getApiClient }} from \"@aptx/api-client\";
// ...
"
)

// 生成 react-query terminal
format!(
    "import {{ getApiClient }} from \"@aptx/api-client\";
// ...
"
)
```

### 阶段 3: 提供初始化脚本

```typescript
// packages/frontend-tk-cli/src/init.ts
export async function initApiClient(adapter: 'axios' | 'uniapp' | 'fetch') {
  const template = await getTemplate(adapter);
  await writeFile('src/api/client.ts', template);
  console.log('ApiClient initialized. Remember to call registerApiClient() in your app init.');
}
```

---

## 5. Monorepo 场景支持

### 目录结构

```
my-monorepo/
├── packages/
│   ├── shared/
│   │   └── api-client/       # 共享的 client 实现
│   │       ├── client.ts
│   │       └── registry.ts
│   ├── web-frontend/         # React 项目
│   │   └── generated/
│   │       └── api/
│   └── mobile-frontend/      # UniApp 项目
│       └── generated/
│           └── api/
```

### 配置支持

用户应通过 CLI 参数传递 client 导入路径，例如：

```bash
aptx-ft codegen --client-import "@my-org/shared/api-client"
```

或指定使用现有 client：

```bash
aptx-ft codegen --use-existing-client
```

---

## 6. 迁移路径

### 对于现有项目

1. 安装 `@aptx/api-client`
2. 创建 `src/api/client.ts` 并注册适配器
3. 重新运行生成命令
4. 删除旧的 `client/` 目录（如果有）

### 对于新项目

1. 运行 `aptx-ft init` 自动设置
2. 选择 HTTP 适配器
3. 开始使用

---

## 7. 决策建议

**推荐方案**: **方案 A (独立 npm 包) + 方案 C 的初始化体验**

**理由**:
1. 独立包确保代码一致性和可维护性
2. 提供多种适配器覆盖主流场景
3. 通过初始化脚本提供良好上手体验
4. 支持高级用户自定义和 monorepo 场景

**下一步行动**:
1. 创建 `@aptx/api-client` 包
2. 实现核心接口和 Axios 适配器
3. 修改生成器引用路径
4. 编写使用文档
5. 添加单元测试

---

## 附录: 代码示例

### Axios 适配器示例

```typescript
// @aptx/api-client/adapters/axios.ts
import axios, { AxiosInstance } from 'axios';
import type { ApiClient, ApiSpec, PerCallOptions } from '../client';

export function createAxiosClient(
  instance: AxiosInstance = axios.create()
): ApiClient {
  return {
    async execute<T>(spec: ApiSpec, options?: PerCallOptions): Promise<T> {
      const response = await instance.request({
        method: spec.method,
        url: spec.path,
        params: spec.query,
        data: spec.body,
        headers: {
          ...spec.headers,
          ...options?.headers,
        },
        signal: options?.signal,
        timeout: options?.timeout,
      });
      return response.data;
    },
  };
}
```

### 应用初始化示例

```typescript
// src/app-init.ts
import { registerApiClient } from '@aptx/api-client';
import { createAxiosClient } from '@aptx/api-client/adapters/axios';
import axios from 'axios';

const axiosInstance = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL,
});

// 添加拦截器
axiosInstance.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  }
);

registerApiClient(createAxiosClient(axiosInstance));
```
