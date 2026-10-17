#!/usr/bin/env node
/**
 * `scripts/compare-artifacts.mjs` 的自测。
 *
 * 这个闸门是「产物未被波及」这类结论的唯一证据来源，它自己出错（漏报 / 误报 / 参数写错
 * 就静默跳过比较）会直接让证据失效，所以必须有回归网。测试一律通过真实的子进程调用，
 * 断言退出码与 stderr/stdout，而不是只单测内部函数——参数的坑恰恰在进程边界上。
 *
 * 运行：`node --test scripts/compare-artifacts.test.mjs`（或 `pnpm test:scripts`）
 */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { after, test } from "node:test";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const gate = path.join(scriptDir, "compare-artifacts.mjs");

const tempRoots = [];
after(() => {
  for (const root of tempRoots) fs.rmSync(root, { recursive: true, force: true });
});

/** 建一棵临时产物树；`files` 为 相对路径 -> 内容。 */
function makeTree(files) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "compare-artifacts-"));
  tempRoots.push(root);
  for (const [relative, content] of Object.entries(files)) {
    const absolute = path.join(root, relative);
    fs.mkdirSync(path.dirname(absolute), { recursive: true });
    fs.writeFileSync(absolute, content, "utf8");
  }
  return root;
}

function runGate(args) {
  const result = spawnSync(process.execPath, [gate, ...args], { encoding: "utf8" });
  return { code: result.status, stdout: result.stdout, stderr: result.stderr };
}

const BASE_FILE = 'export interface Foo {\n  id?: string\n}\n';

test("两棵完全相同的树通过", () => {
  const before = makeTree({ "a/Foo.ts": BASE_FILE, "a/manifest.json": "{}\n" });
  const after = makeTree({ "a/Foo.ts": BASE_FILE, "a/manifest.json": "{}\n" });
  const { code, stdout } = runGate([before, after]);
  assert.equal(code, 0);
  assert.match(stdout, /逐字节一致/);
});

test("删行即使开了 --allow-doc-comments 也失败", () => {
  const before = makeTree({ "a/Foo.ts": BASE_FILE });
  const after = makeTree({ "a/Foo.ts": 'export interface Foo {\n}\n' });
  const { code, stderr } = runGate([before, after, "--allow-doc-comments"]);
  assert.equal(code, 1);
  assert.match(stderr, /删除了 1 行/);
});

test("改行不是「新增注释」，必须失败", () => {
  const before = makeTree({ "a/Foo.ts": 'export type X = string\n' });
  const after = makeTree({ "a/Foo.ts": 'export type X = number\n' });
  const { code, stderr } = runGate([before, after, "--allow-doc-comments"]);
  assert.equal(code, 1);
  assert.match(stderr, /不是允许的文档注释/);
});

test("单行 doc 注释新增：无标志失败，带标志通过", () => {
  const before = makeTree({ "a/Foo.ts": BASE_FILE });
  const after = makeTree({
    "a/Foo.ts": 'export interface Foo {\n  /** 启用标识 */\n  id?: string\n}\n',
  });

  const strict = runGate([before, after]);
  assert.equal(strict.code, 1);
  assert.match(strict.stderr, /新增了 1 行/);

  const allowed = runGate([before, after, "--allow-doc-comments"]);
  assert.equal(allowed.code, 0, allowed.stderr);
  assert.match(allowed.stdout, /1 个文件仅有允许的注释新增/);
});

test("多行 doc 注释块整体放行", () => {
  const before = makeTree({ "a/Foo.ts": BASE_FILE });
  const after = makeTree({
    "a/Foo.ts": [
      "export interface Foo {",
      "  /**",
      "   * 启用标识",
      "   *",
      "   * 第二段说明",
      "   */",
      "  id?: string",
      "}",
      "",
    ].join("\n"),
  });
  const { code, stdout } = runGate([before, after, "--allow-doc-comments"]);
  assert.equal(code, 0, `期望多行注释被放行，实际: ${stdout}`);
});

test("语义化注释被拒绝：@deprecated / \"use client\" / 反引号", () => {
  for (const comment of ['/** @deprecated */', '/** "use client" */', "/** `foo` */"]) {
    const before = makeTree({ "a/Foo.ts": BASE_FILE });
    const after = makeTree({
      "a/Foo.ts": `export interface Foo {\n  ${comment}\n  id?: string\n}\n`,
    });
    const { code, stderr } = runGate([before, after, "--allow-doc-comments"]);
    assert.equal(code, 1, `期望 ${comment} 被拒绝`);
    assert.match(stderr, /不是允许的文档注释/);
  }
});

test("语义化注释混在多行块里同样被拒绝", () => {
  const before = makeTree({ "a/Foo.ts": BASE_FILE });
  const after = makeTree({
    "a/Foo.ts": [
      "export interface Foo {",
      "  /**",
      "   * @deprecated 用 Bar 代替",
      "   */",
      "  id?: string",
      "}",
      "",
    ].join("\n"),
  });
  const { code } = runGate([before, after, "--allow-doc-comments"]);
  assert.equal(code, 1);
});

test("文件缺失或多出都失败", () => {
  const before = makeTree({ "a/Foo.ts": BASE_FILE });
  const after = makeTree({ "a/Foo.ts": BASE_FILE, "a/Extra.ts": BASE_FILE });
  const added = runGate([before, after]);
  assert.equal(added.code, 1);
  assert.match(added.stderr, /多出/);

  const removed = runGate([after, before]);
  assert.equal(removed.code, 1);
  assert.match(removed.stderr, /缺少/);
});

test(".generated/ 默认跳过，--include-generated 时参与比较", () => {
  const before = makeTree({
    "a/Foo.ts": BASE_FILE,
    "a/.generated/manifest.json": '{"generated_at":"2026-01-01"}\n',
  });
  const after = makeTree({
    "a/Foo.ts": BASE_FILE,
    "a/.generated/manifest.json": '{"generated_at":"2026-10-18"}\n',
  });

  const skipped = runGate([before, after]);
  assert.equal(skipped.code, 0, skipped.stderr);

  const included = runGate([before, after, "--include-generated"]);
  assert.equal(included.code, 1);
  assert.match(included.stderr, /generated_at/);
});

test("--report 写文件：`--report <file>` 与 `--report=<file>` 等价", () => {
  const before = makeTree({ "a/Foo.ts": BASE_FILE });
  const after = makeTree({ "a/Foo.ts": BASE_FILE });
  for (const style of ["space", "equals"]) {
    const report = path.join(os.tmpdir(), `compare-artifacts-report-${style}-${process.pid}.txt`);
    const args =
      style === "space" ? [before, after, "--report", report] : [before, after, `--report=${report}`];
    const { code, stdout } = runGate(args);
    assert.equal(code, 0, stdout);
    assert.ok(fs.existsSync(report), `--report（${style}）应写出报告`);
    assert.match(fs.readFileSync(report, "utf8"), /文件数/);
    fs.rmSync(report, { force: true });
  }
});

test("--report 缺值时报用法错误（exit 2），不能吃掉位置参数", () => {
  const before = makeTree({ "a/Foo.ts": BASE_FILE });
  const after = makeTree({ "a/Foo.ts": BASE_FILE });

  // `--report` 落在末尾，压根没有值。
  const { code, stderr } = runGate([before, after, "--report"]);
  assert.equal(code, 2);
  assert.match(stderr, /`--report` 需要一个文件路径参数/);

  // 曾经的缺陷：`--report` 盲目吃掉下一个 token，于是两个位置参数只剩一个，
  // 比较根本没发生，调用方却只看到一行用法提示。现在必须明确报错。
  const greedy = runGate(["--report", before, after]);
  assert.equal(greedy.code, 2);
  assert.match(greedy.stderr, /需要两个位置参数/);
});

test("未知选项报错，不能被静默忽略", () => {
  const before = makeTree({ "a/Foo.ts": BASE_FILE });
  const after = makeTree({ "a/Foo.ts": 'export interface Foo {\n  /** 说明 */\n  id?: string\n}\n' });
  // `--allow-doc-comment`（少个 s）曾会被静默丢掉，然后报出一堆看起来像产物漂移的违规。
  const { code, stderr } = runGate([before, after, "--allow-doc-comment"]);
  assert.equal(code, 2);
  assert.match(stderr, /未知选项/);
});

test("--help 打印用法并 exit 0", () => {
  const { code, stdout } = runGate(["--help"]);
  assert.equal(code, 0);
  assert.match(stdout, /--include-generated/);
  assert.match(stdout, /--report/);
  assert.match(stdout, /退出码/);
});

test("位置参数数量不对时报用法错误", () => {
  for (const args of [[], ["only-one"], ["a", "b", "c"]]) {
    const { code, stderr } = runGate(args);
    assert.equal(code, 2, `args=${JSON.stringify(args)}`);
    assert.match(stderr, /位置参数/);
  }
});

test("读不到目录时以 exit 1 失败（区别于用法错误）", () => {
  const before = makeTree({ "a/Foo.ts": BASE_FILE });
  const { code, stderr } = runGate([before, path.join(os.tmpdir(), "definitely-missing-dir-xyz")]);
  assert.equal(code, 1);
  assert.match(stderr, /不是目录/);
});
