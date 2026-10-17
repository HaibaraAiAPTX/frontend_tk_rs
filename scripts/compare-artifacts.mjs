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
 *   node scripts/compare-artifacts.mjs <before> <after> --include-generated
 *   node scripts/compare-artifacts.mjs --help
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
 * `--allow-doc-comments` 用于「只允许新增注释」的改动（例如补齐布尔属性描述）。它仍然
 * 禁止删行、改行，也禁止注释以外的任何新增；单行与多行文档注释块都接受，但含 `@`、引号
 * 或反引号的注释会被拒绝（宁可误报，也不放行 `@deprecated`、`"use client"` 这类有语义的
 * 注释）。边界见 `docs/llm/frontend_tk_rs_codegen.md` §5。
 *
 * 退出码：
 *   0 = 完全一致（或差异全部是允许的注释新增）
 *   1 = 产物有差异，或读不到目录
 *   2 = 参数用法错误（CI 可据此区分「产物漂移」与「调用写错」）
 *
 * 本脚本的行为由 `scripts/compare-artifacts.test.mjs` 覆盖（`node --test`）。
 */
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

// 内嵌时间戳、默认跳过的元数据目录。
const GENERATED_DIR = ".generated";

const USAGE = `用法: node scripts/compare-artifacts.mjs <before> <after> [选项]

选项:
  --allow-doc-comments   允许「新增整行/整块注释」，仍禁止删行、改行与其它新增
  --include-generated    连 .generated/ 下的时间戳元数据一起比较（默认跳过）
  --report <file>        把逐文件比较结果写入文件（也支持 --report=<file>）
  -h, --help             打印本帮助

退出码: 0 = 一致或仅有允许的注释新增；1 = 存在不允许的差异；2 = 参数用法错误`;

/** 参数解析失败时以 exit 2 结束，与「产物有差异」的 exit 1 区分开。 */
function usageError(message) {
  console.error(`参数错误: ${message}`);
  console.error(USAGE);
  process.exit(2);
}

function parseArgs(argv) {
  // 先标出所有「选项」，包括 `--report <value>` 的值位置。位置参数 = 其余 token。
  // 这样 `--report` 无论写在最前还是最后，都不会吃掉位置参数（旧实现盲目取下一个
  // token，`--report <before> <after>` 会静默退化成「只有一个位置参数」）。
  const isOption = new Array(argv.length).fill(false);
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (!arg.startsWith("-") || arg === "-") continue;
    isOption[index] = true;
    if (arg === "--report") {
      const value = argv[index + 1];
      // 缺值时必须报错，不能把下一个 token（可能是位置参数）当成文件名。
      if (value === undefined || value.startsWith("-")) {
        usageError("`--report` 需要一个文件路径参数");
      }
      isOption[index + 1] = true;
      index += 1;
    }
  }

  const options = {
    allowDocComments: false,
    includeGenerated: false,
    reportPath: null,
    positionals: [],
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (!isOption[index]) {
      options.positionals.push(arg);
    } else if (arg === "--allow-doc-comments") {
      options.allowDocComments = true;
    } else if (arg === "--include-generated") {
      options.includeGenerated = true;
    } else if (arg === "-h" || arg === "--help") {
      console.log(USAGE);
      process.exit(0);
    } else if (arg === "--report") {
      options.reportPath = argv[index + 1];
      index += 1;
    } else if (arg.startsWith("--report=")) {
      const value = arg.slice("--report=".length);
      if (value === "") usageError("`--report=` 后面需要文件路径");
      options.reportPath = value;
    } else {
      // 未知选项必须报错：`--allow-doc-comment`（少个 s）之类的笔误若被静默忽略，
      // 只会让调用方看到一堆「新增」违规，却以为是产物漂移。
      usageError(`未知选项 \`${arg}\``);
    }
  }
  if (options.positionals.length !== 2) {
    usageError(`需要两个位置参数（before 与 after），实际收到 ${options.positionals.length} 个`);
  }
  return options;
}

/**
 * 整行注释（缩进任意、允许行内闭合）：`/** 说明 *\/`。
 *
 * 用 [\s\S] 而不是 `.`，以便同一套正则识别块的开头与结尾行。
 */
const SINGLE_LINE_COMMENT = /^\s*\/\*\*[\s\S]*\*\/\s*$/;

/** 块的开头 / 结尾 / 中间续行。 */
const BLOCK_OPEN = /^\s*\/\*\*/;
const BLOCK_CLOSE = /\*\/\s*$/;
/** 多行注释块内部的续行（` * 说明`）；块内不是这个形状的行一律不放行。 */
const BLOCK_CONTINUATION = /^\s*\*/;

/**
 * 含 `@`、单双引号或反引号的注释不当作「纯文档」放行。
 *
 * `/** @deprecated *\/`、`/** "use client" *\/`、`/** `foo` *\/` 都是会被工具链读取语义的
 * 注释，不是说明文字；这个闸门宁可漏报（拒绝一次合法注释新增）也不放行它们。
 */
const SEMANTIC_COMMENT = /[@"'`]/;

/**
 * 判断新增的一行是否属于「允许的文档注释」，并推进块状态。
 *
 * `state.inBlock` 表示上一新增行开了一个尚未闭合的 `/** ...` 块（多行 JSDoc 的常见形态）。
 * 一旦块内出现语义化字符，整个块都会被拒绝——`state.rejected` 会一直带到闭合行。
 */
function classifyAddedLine(line, state) {
  if (state.inBlock) {
    // 块内只接受 ` * …` 续行与闭合行。块开了一半却混进代码（说明块被截断或产物本身
    // 就是坏的）时，这里必须拒绝而不是把整块都当作注释放行。
    if (!BLOCK_CONTINUATION.test(line)) return false;
    if (SEMANTIC_COMMENT.test(line)) state.rejected = true;
    if (BLOCK_CLOSE.test(line)) state.inBlock = false;
    return !state.rejected;
  }
  if (!BLOCK_OPEN.test(line)) return false;
  if (SEMANTIC_COMMENT.test(line)) return false;
  // 单行闭合则块已结束（上面的行内检查已覆盖语义化内容）。
  if (!BLOCK_CLOSE.test(line)) state.inBlock = true;
  return true;
}

// 供测试直接引用；作为脚本运行时 argv[1] 就是本文件。
export { classifyAddedLine, parseArgs };

const isMain =
  process.argv[1] !== undefined &&
  import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href;


if (isMain) {
  const { allowDocComments, includeGenerated, reportPath, positionals } = parseArgs(
    process.argv.slice(2),
  );

  const beforeRoot = path.resolve(positionals[0]);
  const afterRoot = path.resolve(positionals[1]);

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
    // 块状态跨行延续：多行 JSDoc 的第一行不闭合，后续续行必须一并接受。
    const commentState = { inBlock: false, rejected: false };

    const recordAdded = (line) => {
      result.added += 1;
      if (!classifyAddedLine(line, commentState)) result.otherAdded.push(line);
    };

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
        recordAdded(afterLines[j]);
        j += 1;
      }
    }
    for (; i < rows; i += 1) {
      result.removed += 1;
      result.otherRemoved.push(beforeLines[i]);
    }
    for (; j < cols; j += 1) {
      recordAdded(afterLines[j]);
    }
    return result;
  }

  const lines = [];
  lines.push(`before: ${beforeRoot}`);
  lines.push(`after : ${afterRoot}`);
  lines.push(`文件数: ${before.size} -> ${after.size}`);
  lines.push(`允许注释新增: ${allowDocComments ? "是" : "否"}`);
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
          `${relative}: ${otherAdded.length} 行新增不是允许的文档注释（首个: ${JSON.stringify(otherAdded[0])}）`,
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
}
