# APTX 代码生成模板契约（Draft）

状态：Draft  
日期：2026-02-09  
关联文档：`docs/next-gen-codegen-architecture.md`、`docs/codegen-execution-plan.md`

## 1. 作用域

本文件仅定义模板层契约：

1. 模板输入数据结构。
2. 模板输出目录与命名规则。
3. 关键行为约束（queryKey/signal/meta 等）。

不覆盖：CLI 设计、插件加载、执行计划。

## 2. 模板输入契约

```ts
type GeneratorInput = {
  project: {
    packageName: string;
    apiBasePath?: string;
    terminals: Array<"functions" | "react-query" | "vue-query">;
    retryOwnership?: "core" | "query" | "hybrid";
  };
  endpoints: EndpointItem[];
};

type EndpointItem = {
  namespace: string[];
  operationName: string;
  method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
  path: string;
  inputTypeName: string;
  outputTypeName: string;
  requestBodyField?: string;
  queryFields?: string[];
  pathFields?: string[];
  hasRequestOptions: boolean;
  supportsQuery: boolean;
  supportsMutation: boolean;
  deprecated?: boolean;
};
```

硬规则：模板层不得再解析 OpenAPI 原始结构。

## 3. 输出目录契约

```txt
generated/
  spec/
    endpoints/**/*
    types/**/*
  client/
    createClient.ts
    registry.ts
  api/
    functions/**/*
    react-query/**/*      (可选)
    vue-query/**/*        (可选)
```

硬规则：

1. one-endpoint-per-file。
2. index 仅做 re-export，不导出聚合对象。
3. terminal 之间禁止互相 import。

## 4. 终端符号契约

## 4.1 functions

每个 endpoint 必须生成：

1. `xxx(input, options?) => Promise<Output>`
2. 仅依赖 `getApiClient()`、`buildXxxSpec()` 与类型定义。

## 4.2 react-query

query endpoint：

1. `xxxQueryDef`
2. `xxxKey`
3. `useXxxQuery`

mutation endpoint：

1. `xxxMutationDef`
2. `useXxxMutation`

## 4.3 vue-query

与 react-query 对齐：

1. `xxxQueryDef` / `xxxKey` / `useXxxQuery`
2. `xxxMutationDef` / `useXxxMutation`

## 5. 命名与文件规则

1. Query 文件：`{operation}.query.ts`
2. Mutation 文件：`{operation}.mutation.ts`
3. Functions 文件：`{operation}.ts`
4. Spec Builder：`build{PascalOperation}Spec`
5. Hook 命名：
   - `use{PascalOperation}Query`
   - `use{PascalOperation}Mutation`

## 6. 行为契约

1. Query `signal` 必须透传到 `PerCallOptions.signal`。
2. Query `meta` 必须透传（建议 `meta.__query`）。
3. `queryKey` 必须稳定（`keyPrefix + normalizedInput`）。
4. GET 默认走 query；非 GET 默认走 mutation。
5. `deprecated` endpoint 生成注释但不阻止产出。

## 7. 示例输入与输出

## 7.1 示例输入（JSON）

```json
{
  "project": {
    "packageName": "@demo/api",
    "terminals": ["functions", "react-query", "vue-query"],
    "retryOwnership": "core"
  },
  "endpoints": [
    {
      "namespace": ["account-category"],
      "operationName": "getInfo",
      "method": "GET",
      "path": "/AccountCategory/GetInfo",
      "inputTypeName": "AccountCategoryGetInfoInput",
      "outputTypeName": "AccountCategoryDTOResultModel",
      "queryFields": ["id"],
      "hasRequestOptions": true,
      "supportsQuery": true,
      "supportsMutation": false
    },
    {
      "namespace": ["account-category"],
      "operationName": "edit",
      "method": "PUT",
      "path": "/AccountCategory/Edit",
      "inputTypeName": "EditAccountCategoryRequestModel",
      "outputTypeName": "ResultModel",
      "requestBodyField": "data",
      "hasRequestOptions": true,
      "supportsQuery": false,
      "supportsMutation": true
    }
  ]
}
```

## 7.2 functions/getInfo

`generated/api/functions/account-category/getInfo.ts`

```ts
import type { PerCallOptions } from "../../../client/client";
import { getApiClient } from "../../../client/registry";
import { buildGetInfoSpec } from "../../../../spec/endpoints/account-category/getInfo";
import type { AccountCategoryGetInfoInput } from "../../../../spec/types/AccountCategoryGetInfoInput";
import type { AccountCategoryDTOResultModel } from "../../../../spec/types/AccountCategoryDTOResultModel";

export function getInfo(
  input: AccountCategoryGetInfoInput,
  options?: PerCallOptions
): Promise<AccountCategoryDTOResultModel> {
  return getApiClient().execute<AccountCategoryDTOResultModel>(buildGetInfoSpec(input), options);
}
```

## 7.3 react-query/getInfo.query

`generated/api/react-query/account-category/getInfo.query.ts`

```ts
import { createQueryDefinition } from "@aptx/api-query-adapter";
import { createReactQueryHooks } from "@aptx/api-query-react";
import { getApiClient } from "../../../client/registry";
import { buildGetInfoSpec } from "../../../../spec/endpoints/account-category/getInfo";
import type { AccountCategoryGetInfoInput } from "../../../../spec/types/AccountCategoryGetInfoInput";
import type { AccountCategoryDTOResultModel } from "../../../../spec/types/AccountCategoryDTOResultModel";

export const getInfoQueryDef = createQueryDefinition<AccountCategoryGetInfoInput, AccountCategoryDTOResultModel>({
  keyPrefix: ["account-category", "getInfo"] as const,
  buildSpec: buildGetInfoSpec,
  execute: (spec, options) => getApiClient().execute(spec, options),
});

export const getInfoKey = getInfoQueryDef.key;

export const { useAptxQuery: useGetInfoQuery } = createReactQueryHooks(getInfoQueryDef);
```

## 7.4 vue-query/getInfo.query

`generated/api/vue-query/account-category/getInfo.query.ts`

```ts
import { createQueryDefinition } from "@aptx/api-query-adapter";
import { createVueQueryHooks } from "@aptx/api-query-vue";
import { getApiClient } from "../../../client/registry";
import { buildGetInfoSpec } from "../../../../spec/endpoints/account-category/getInfo";
import type { AccountCategoryGetInfoInput } from "../../../../spec/types/AccountCategoryGetInfoInput";
import type { AccountCategoryDTOResultModel } from "../../../../spec/types/AccountCategoryDTOResultModel";

export const getInfoQueryDef = createQueryDefinition<AccountCategoryGetInfoInput, AccountCategoryDTOResultModel>({
  keyPrefix: ["account-category", "getInfo"] as const,
  buildSpec: buildGetInfoSpec,
  execute: (spec, options) => getApiClient().execute(spec, options),
});

export const getInfoKey = getInfoQueryDef.key;

export const { useAptxQuery: useGetInfoQuery } = createVueQueryHooks(getInfoQueryDef);
```

## 7.5 react-query/edit.mutation

`generated/api/react-query/account-category/edit.mutation.ts`

```ts
import { createMutationDefinition } from "@aptx/api-query-adapter";
import { createReactMutationHooks } from "@aptx/api-query-react";
import { getApiClient } from "../../../client/registry";
import { buildEditSpec } from "../../../../spec/endpoints/account-category/edit";
import type { EditAccountCategoryRequestModel } from "../../../../spec/types/EditAccountCategoryRequestModel";
import type { ResultModel } from "../../../../spec/types/ResultModel";

export const editMutationDef = createMutationDefinition<EditAccountCategoryRequestModel, ResultModel>({
  buildSpec: buildEditSpec,
  execute: (spec, options) => getApiClient().execute(spec, options),
});

export const { useAptxMutation: useEditMutation } = createReactMutationHooks(editMutationDef);
```

## 8. 自检清单（CI）

1. 模板快照测试（固定输入 -> 固定输出）。
2. 生成产物 `tsc --noEmit`。
3. React/Vue 示例工程构建。
4. lint 规则校验（禁止 terminal 互相依赖）。
