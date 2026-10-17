---
"@aptx-js/frontend-tk-core": patch
---

产物回归闸门：`scripts/compare-artifacts.mjs`（`pnpm compare:artifacts`；不用 `verify:artifacts`
这个名字，避免和发布闸门 `scripts/verify-artifacts.mjs` 混淆——那个由 `release.yml` 直接调用，
管的是 9 个平台 `.node` 是否就位）。

改动 parser / renderer / 各语言映射时，「产物没有被波及」必须拿出可复现的证据，而不是只凭单测
和人工抽查。这个脚本逐字节比较两棵由同一组 spec 生成的产物树：

- 默认口径是**逐字节一致**，删行、改行、多出或缺失文件都直接判失败；
- `--allow-doc-comments` 用于「只允许新增注释」的改动（例如补齐属性描述），仍拒绝删行、
  改行以及注释以外的任何新增；单行 `/** 说明 */` 与多行 `/** … */` 块都接受，但含 `@`、
  引号或反引号的注释一律拒绝（`/** @deprecated */`、`/** "use client" */` 带语义，宁可误报）；
- 默认跳过 `.generated/`（`manifest.json` / `deletion-report.*` 内嵌 `generated_at` 时间戳，
  两次运行必然不同），需要时用 `--include-generated` 关掉这个跳过；
- `--report <file>` 输出逐文件差异，便于附到 issue / PR；
- 参数写错（未知选项、`--report` 缺值）以 **exit 2** 报用法错误，与「产物有差异」的
  **exit 1** 区分开，CI 可直接据此判断；`--help` 打印用法。

闸门自身由 `scripts/compare-artifacts.test.mjs` 覆盖（`pnpm test:scripts`，16 个用例，走真实
子进程断言退出码与输出）：一致 / 删行 / 改行 / 单行与多行注释 / 语义化注释拒收 / 文件多出缺失 /
`.generated/` 跳过 / 参数与退出码。多行注释块**只放行 ` * ` 续行与 `*/` 闭合行**，块开了却混进
代码必须拒绝——否则「块内一律算注释」会把真实代码改动静默放过，那是闸门最不能有的漏报。

已用它复核本次修复：以 `47c7277` 与修复后各构建一次、13 份真实 spec 全量对拍，得到
TypeScript 123 个文件 / 181 行纯新增注释（零删除），Python **逐字节一致**。

生成两棵树的步骤、13 份真实 spec 的规模与验收口径见 `docs/llm/frontend_tk_rs_codegen.md` §5
与 `docs/llm/frontend_tk_rs_model_ir.md` §6。
