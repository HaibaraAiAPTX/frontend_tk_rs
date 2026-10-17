---
"@aptx-js/frontend-tk-core": patch
---

产物回归闸门：`scripts/compare-artifacts.mjs`（`pnpm verify:artifacts`）。

改动 parser / renderer / 各语言映射时，「产物没有被波及」必须拿出可复现的证据，而不是只凭单测
和人工抽查。这个脚本逐字节比较两棵由同一组 spec 生成的产物树：

- 默认口径是**逐字节一致**，删行、改行、多出或缺失文件都直接判失败；
- `--allow-doc-comments` 用于「只允许新增整行注释」的改动（例如补齐属性描述），仍拒绝删行、
  改行以及注释以外的任何新增；
- 默认跳过 `.generated/`（`manifest.json` / `deletion-report.*` 内嵌 `generated_at` 时间戳，
  两次运行必然不同），需要时用 `--include-generated` 关掉这个跳过；
- `--report <file>` 输出逐文件差异，便于附到 issue / PR。

已用它复核本次修复：以 `47c7277` 与修复后各构建一次、13 份真实 spec 全量对拍，得到
TypeScript 123 个文件 / 181 行纯新增注释（零删除），Python **逐字节一致**。

生成两棵树的步骤、13 份真实 spec 的规模与验收口径见 `docs/llm/frontend_tk_rs_codegen.md` §5
与 `docs/llm/frontend_tk_rs_model_ir.md` §6。
