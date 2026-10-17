#!/usr/bin/env node
/**
 * 产物回归闸门：逐字节比较两棵由同一组 OpenAPI spec 生成的产物树。
 *
 * 为什么需要它：模型管线（`swagger_tk` / `swagger_gen` / 各语言 terminal）的改动只要碰到
 * parser 或 renderer，就必须证明「13 份真实 spec、1121 个模型」的产物没有被波及。
 * 这个结论靠手工对拍得不出可复现的证据，也不该只留在提交信息里。
 *
 * 用法（`<before>` 与 `<after>` 是两次生成各自的输出根目录，子目录名需一一对应）：
 *
 *   node scripts/compare-artifacts.mjs <before> <after>
 *   node scripts/compare-artifacts.mjs <before> <after> --allow-doc-comments
 *   node scripts/compare-artifacts.mjs <before> <after> --report report.txt
 *
 * 默认跳过 `.generated/`（`manifest.json` / `deletion-report.*`）：它们内嵌 `generated_at`
 * 时间戳，两次运行必然不同，属于确定性噪声而不是产物回归。确实要比它们时加
 * `--include-generated`。
 *
 * 生成两棵树的最小流程（在 `frontend_tk_rs` 根目录）：
 *
 *   pnpm build                                   # 构建 Rust binding + CLI
 *   node packages/frontend-tk-cli/bin/aptx.js model gen \
 *     --input <spec.json> --output <after>/<spec> --style module
 *
 * 退出码：0 = 完全一致（或差异全部是允许的注释新增）；1 = 存在不允许的差异或读不到目录。
 *
 * `--allow-doc-comments` 用于「只允许新增整行注释」的改动（例如补齐布尔属性描述）。
 * 它仍然禁止删行、改行，也禁止注释以外的任何新增。
 */
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const args = process.argv.slice(2);
const allowDocComments = args.includes("--allow-doc-comments");
const includeGenerated = args.includes("--include-generated");
const reportIndex = args.indexOf("--report");
const reportPath = reportIndex === -1 ? null : args[reportIndex + 1];
const positional = args.filter((arg, index) => {
  if (arg.startsWith("--")) return false;
  return !(reportIndex !== -1 && index === reportIndex + 1);
});

if (positional.length !== 2) {
  console.error(
    "用法: node scripts/compare-artifacts.mjs <before> <after> [--allow-doc-comments] [--include-generated] [--report <file>]",
  );
  process.exit(1);
}

const beforeRoot = path.resolve(positional[0]);
const afterRoot = path.resolve(positional[1]);

// 整行 doc 注释（缩进任意，允许行内闭合）。
const DOC_COMMENT_LINE = /^\s*\/\*\*[\s\S]*\*\/\s*$/;

// 内嵌时间戳、默认跳过的元数据目录。
const GENERATED_DIR = ".generated";

function fail(message, details = []) {
  console.error(`\n[FAIL] ${message}`);
  for (const detail of details.slice(0, 50)) console.error(`  - ${detail}`);
  if (details.length > 50) console.error(`  … 另有 ${details.length} 条`);
  process.exit(1);
}

/** 相对路径 -> sha256，用于先筛掉未变文件，避免逐行读整棵树。 */
function digestTree(root, label) {
  if (!fs.existsSync(root) || !fs.statSync(root).isDirectory()) {
    fail(`${label} 不是目录: ${root}`);
  }
  const digests = new Map();
  const walk = (dir, prefix) => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const absolute = path.join(dir, entry.name);
      const relative = prefix === "" ? entry.name : `${prefix}/${entry.name}`;
      if (entry.isDirectory()) {
        const segments = relative.split("/");
        if (!includeGenerated && segments.includes(GENERATED_DIR)) continue;
        walk(absolute, relative);
      } else if (entry.isFile()) {
        const content = fs.readFileSync(absolute);
        digests.set(relative, crypto.createHash("sha256").update(content).digest("hex"));
      }
    }
  };
  walk(root, "");
  return digests;
}

const before = digestTree(beforeRoot, "before");
const after = digestTree(afterRoot, "after");

if (before.size === 0 && after.size === 0) {
  fail("两棵树都是空的，无法比较");
}

const onlyBefore = [...before.keys()].filter((key) => !after.has(key)).sort();
const onlyAfter = [...after.keys()].filter((key) => !before.has(key)).sort();
const changed = [...before.keys()]
  .filter((key) => after.has(key) && before.get(key) !== after.get(key))
  .sort();

/** 逐行比对单个文件；`added`/`removed` 为总行数，`otherAdded` 记录「不算注释」的新增行。 */
function diffLines(relative) {
  const beforeLines = fs.readFileSync(path.join(beforeRoot, relative), "utf8").split("\n");
  const afterLines = fs.readFileSync(path.join(afterRoot, relative), "utf8").split("\n");
  const result = { added: 0, removed: 0, otherAdded: [], otherRemoved: [] };

  // 简单 LCS：产物是生成出来的、差异通常很小，够用且不引依赖。
  const rows = beforeLines.length;
  const cols = afterLines.length;
  const table = Array.from({ length: rows + 1 }, () => new Uint32Array(cols + 1));
  for (let i = rows - 1; i >= 0; i -= 1) {
    for (let j = cols - 1; j >= 0; j -= 1) {
      table[i][j] =
        beforeLines[i] === afterLines[j]
          ? table[i + 1][j + 1] + 1
          : Math.max(table[i + 1][j], table[i][j + 1]);
    }
  }

  let i = 0;
  let j = 0;
  while (i < rows && j < cols) {
    if (beforeLines[i] === afterLines[j]) {
      i += 1;
      j += 1;
    } else if (table[i + 1][j] >= table[i][j + 1]) {
      result.removed += 1;
      result.otherRemoved.push(beforeLines[i]);
      i += 1;
    } else {
      result.added += 1;
      if (!DOC_COMMENT_LINE.test(afterLines[j])) result.otherAdded.push(afterLines[j]);
      j += 1;
    }
  }
  for (; i < rows; i += 1) {
    result.removed += 1;
    result.otherRemoved.push(beforeLines[i]);
  }
  for (; j < cols; j += 1) {
    result.added += 1;
    if (!DOC_COMMENT_LINE.test(afterLines[j])) result.otherAdded.push(afterLines[j]);
  }
  return result;
}

const lines = [];
lines.push(`before: ${beforeRoot}`);
lines.push(`after : ${afterRoot}`);
lines.push(`文件数: ${before.size} -> ${after.size}`);
lines.push(`允许整行 doc 注释新增: ${allowDocComments ? "是" : "否"}`);
lines.push(`包含 .generated 时间戳文件: ${includeGenerated ? "是" : "否"}`);
lines.push("");

for (const relative of onlyBefore) lines.push(`[缺少] after 中没有: ${relative}`);
for (const relative of onlyAfter) lines.push(`[多出] before 中没有: ${relative}`);

let addedTotal = 0;
let removedTotal = 0;
const violations = [
  ...onlyBefore.map((key) => `缺少 ${key}`),
  ...onlyAfter.map((key) => `多出 ${key}`),
];

for (const relative of changed) {
  const { added, removed, otherAdded, otherRemoved } = diffLines(relative);
  addedTotal += added;
  removedTotal += removed;
  lines.push(`[变更] ${relative}: +${added} / -${removed}`);
  if (removed > 0) {
    violations.push(
      `${relative}: 删除了 ${removed} 行（首行: ${JSON.stringify(otherRemoved[0] ?? "")}）`,
    );
  }
  if (allowDocComments) {
    if (otherAdded.length > 0) {
      violations.push(
        `${relative}: ${otherAdded.length} 行新增不是整行注释（首个: ${JSON.stringify(otherAdded[0])}）`,
      );
    }
  } else if (added > 0) {
    violations.push(`${relative}: 新增了 ${added} 行`);
  }
}

lines.push("");
lines.push(`变更文件: ${changed.length}  新增行: ${addedTotal}  删除行: ${removedTotal}`);

const report = `${lines.join("\n")}\n`;
if (reportPath) {
  fs.writeFileSync(path.resolve(reportPath), report, "utf8");
  console.log(`比较报告已写入 ${path.resolve(reportPath)}`);
} else {
  process.stdout.write(report);
}

if (violations.length > 0) {
  fail(`${violations.length} 处差异不在允许范围内`, violations);
}
console.log(
  allowDocComments
    ? `[OK] ${before.size} 个文件一致，${changed.length} 个文件仅有允许的注释新增`
    : `[OK] ${before.size} 个文件逐字节一致`,
);
