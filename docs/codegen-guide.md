# Frontend TK 代码生成器使用指南（更新版）

状态：Active  
日期：2026-02-14  
适用仓库：`frontend_tk_rs`

---

## 1. 快速开始

安装：

```bash
pnpm add -D @aptx/frontend-tk-cli
```

下载 OpenAPI（可选）：

```bash
aptx-ft input download --url https://example.com/openapi.json --output ./openapi.json
```

生成模型：

```bash
aptx-ft -i ./openapi.json model gen --output ./src/models --style module
```

生成 React Query：

```bash
aptx-ft aptx react-query -i ./openapi.json -o ./src/api \
  --client-mode global --model-mode relative --model-path ./src/models
```

---

## 2. 命令总览

当前命令（以 `aptx-ft --help` 为准）：

- `aptx functions`
- `aptx react-query`
- `aptx vue-query`
- `model gen`
- `model ir`
- `model enum-plan`
- `model enum-apply`
- `materal enum-patch`
- `materal enum-plan`
- `materal enum-apply`
- `input download`

说明：
- `codegen run` 已删除，不再作为聚合入口。

---

## 3. aptx 终端命令

## 3.1 `aptx functions`

```bash
aptx-ft aptx functions -i <spec-file-or-url> -o <output-dir> \
  --client-mode global \
  --model-mode relative --model-path <model-dir>
```

## 3.2 `aptx react-query`

```bash
aptx-ft aptx react-query -i <spec-file-or-url> -o <output-dir> \
  --client-mode global \
  --model-mode relative --model-path <model-dir>
```

## 3.3 `aptx vue-query`

```bash
aptx-ft aptx vue-query -i <spec-file-or-url> -o <output-dir> \
  --client-mode global \
  --model-mode relative --model-path <model-dir>
```

---

## 4. model 命令

## 4.1 `model gen`

```bash
aptx-ft -i <spec-file-or-url> model gen --output <dir> --style module
```

可选参数：
- `--name <schema>`（可重复）限制生成范围

## 4.2 `model ir`

```bash
aptx-ft -i <spec-file-or-url> model ir --output ./tmp/model-ir.json
```

## 4.3 `model enum-plan`

```bash
aptx-ft -i <spec-file-or-url> model enum-plan --output ./tmp/enum-plan.json
```

## 4.4 `model enum-apply`

```bash
aptx-ft -i <spec-file-or-url> model enum-apply \
  --patch ./tmp/enum-patch.json \
  --output ./src/models \
  --style module \
  --conflict-policy patch-first
```

---

## 5. materal 命令

## 5.1 `materal enum-patch`

```bash
aptx-ft -i <spec-file-or-url> materal enum-patch \
  --base-url <url> \
  --output ./tmp/enum-patch.json
```

## 5.2 `materal enum-plan`

```bash
aptx-ft -i <spec-file-or-url> materal enum-plan --output ./tmp/enum-plan.json
```

## 5.3 `materal enum-apply`

```bash
aptx-ft -i <spec-file-or-url> materal enum-apply \
  --patch ./tmp/enum-patch.json \
  --output ./src/models \
  --style module
```

---

## 6. 输出结构（当前规范）

`-o ./src/api` 时示例：

```text
src/api/
├─ spec/
│  └─ assignment/
│     └─ add.ts
├─ functions/
│  └─ assignment/
│     └─ add.ts
├─ react-query/
│  └─ assignment/
│     ├─ add.query.ts
│     └─ add.mutation.ts
└─ vue-query/
   └─ assignment/
      ├─ add.query.ts
      └─ add.mutation.ts
```

说明：
- 不再使用历史目录 `spec/endpoints/*` 与 `functions/api/*`。
- 各目录会自动生成 `index.ts` re-export。

---

## 7. 单项目与 Monorepo 参数建议

## 7.1 单项目（`src` 内）

推荐：
- `-o ./src/api`
- `--model-mode relative --model-path ./src/models`
- `--client-mode global`

## 7.2 Monorepo（模型独立包）

推荐：
- `-o ./apps/<app>/src/api`
- `--model-mode package --model-path @org/models`
- `--client-mode package --client-package @org/api-client`（按项目约定）

---

## 8. 导入策略说明

- `--model-mode relative`：生成器按“当前文件 -> 模型目录”动态计算相对路径
- `--model-mode package`：按包名导入
- endpoint/spec 互引同样动态计算，禁止模板硬编码深度

---

## 9. 常见问题

## 9.1 生成后还是旧代码结构

先执行：

```bash
pnpm build
```

再执行 `packages/frontend-tk-cli/bin/aptx.js` 进行端到端验证。

## 9.2 改了代码但 CLI 帮助或行为没变

`aptx.js` 开启 compile cache，复测时可禁用：

```bash
NODE_DISABLE_COMPILE_CACHE=1 node packages/frontend-tk-cli/bin/aptx.js --help
```

---

## 10. 最小验证流程（开发者）

1. 在 `frontend_tk_rs` 根目录执行 `pnpm build`
2. 使用真实命令生成（functions/react-query/vue-query）
3. 检查：
   - 目录结构是否为 `spec/*`、`functions/*`
   - import 是否动态且正确
   - 关键文件无多余空行和错误类型
