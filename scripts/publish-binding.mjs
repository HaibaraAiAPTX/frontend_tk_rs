#!/usr/bin/env node
/**
 * 发布 @aptx-js/frontend-tk-binding 及其 9 个平台包。
 *
 * 跑在 release.yml 的 CI 里，二进制直接来自构建矩阵的 artifact，全程无人介入。
 * 旧版本是给开发者在本地跑的（`gh release download` 回捞二进制），一旦有人
 * 忘记跑、跑错版本、或者少下一个平台文件，正确性就没有保证。
 *
 * 流程（与 napi-rs 官方 release 文档一致）：
 *   1. verify-artifacts  证明 9 个平台的 .node 全部就位且版本一致
 *   2. napi version      同步 9 个平台包的 version 字段
 *   3. napi prepublish   回写根包 optionalDependencies，逐个发布平台包
 *   4. npm publish       发布根包
 *
 * 用法：
 *   node scripts/publish-binding.mjs --dry-run    # 只校验，不发布
 *   node scripts/publish-binding.mjs              # 真实发布
 */
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");
const bindingDir = path.join(repoRoot, "crates/node_binding");
const pkgPath = path.join(bindingDir, "package.json");

const dryRun = process.argv.includes("--dry-run");
const registry = process.env.NPM_REGISTRY ?? "https://registry.npmjs.org/";

function run(cmd, args, opts = {}) {
  const printable = [cmd, ...args].join(" ");
  console.log(`\n$ ${printable}`);
  const result = spawnSync(cmd, args, {
    cwd: opts.cwd ?? bindingDir,
    stdio: "inherit",
    shell: process.platform === "win32",
    env: { ...process.env, ...opts.env },
  });
  if (result.status !== 0) {
    throw new Error(`命令失败（exit ${result.status}）: ${printable}`);
  }
}

function main() {
  const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf8"));
  const version = pkg.version;
  console.log(`发布 ${pkg.name}@${version} 到 ${registry}`);
  if (dryRun) console.log("（dry-run：只做校验，不会发布任何东西）\n");

  // 1. 闸门：任何平台缺失或版本不一致都必须在这里失败，而不是让 napi 静默跳过
  run("node", [path.join(repoRoot, "scripts/verify-artifacts.mjs"), "--verbose"], {
    cwd: repoRoot,
  });

  // dry-run 到这一步就够了：napi prepublish 的 --dry-run 是静默空操作
  // （实测无输出、不回写 optionalDependencies），继续跑没有验证价值。
  if (dryRun) {
    console.log("\n[dry-run] 校验通过，跳过版本同步与发布");
    return;
  }

  // 2. 同步平台包版本号。注意 napi version 只改平台包的 version 字段，
  //    不会动根包的 optionalDependencies —— 那由第 3 步的 prepublish 负责。
  run("npx", ["napi", "version"]);

  // 3. 回写 optionalDependencies 并逐个发布平台包。
  //    缺失的 target 在这里只会 warning 不会失败，所以第 1 步的闸门是必需的。
  //    scoped 包默认按 private 处理，必须显式 --access public 才能发到公共 registry。
  run("npx", ["napi", "prepublish", "-t", "npm", "--no-gh-release"], {
    env: { NPM_CONFIG_REGISTRY: registry, NPM_CONFIG_ACCESS: "public" },
  });

  // 4. 发布根包。--ignore-scripts 避免 prepublishOnly 钩子再触发一轮平台包发布。
  run(
    "pnpm",
    ["publish", "--no-git-checks", "--ignore-scripts", "--access", "public", "--registry", registry],
    { cwd: bindingDir },
  );

  console.log(`\n完成：${pkg.name}@${version}`);
}

try {
  main();
} catch (error) {
  console.error(`\n[FAIL] ${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
}
