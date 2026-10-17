#!/usr/bin/env node
/**
 * 发布前校验：git tag 与所有待发布包的版本必须一致。
 *
 * 为什么需要：`napi prepublish -t npm` 用 `v<package.json version>` 作为
 * GitHub Release 的 tag 名。如果 tag 是 v0.1.20 而 package.json 还停在
 * 0.1.21（或反过来），会创建一个对不上的 release，而且 npm 版本不可覆盖。
 *
 * 用法: node scripts/verify-version.mjs v0.1.20
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");

const rawTag = process.argv[2];
if (!rawTag) {
  console.error("用法: node scripts/verify-version.mjs <tag>，例如 v0.1.20");
  process.exit(1);
}
const tagVersion = rawTag.replace(/^v/, "");

const packages = [];

// packages/* 下的可发布包
const packagesDir = path.join(repoRoot, "packages");
for (const d of fs.readdirSync(packagesDir, { withFileTypes: true })) {
  if (!d.isDirectory()) continue;
  const p = path.join(packagesDir, d.name, "package.json");
  if (!fs.existsSync(p)) continue;
  const j = JSON.parse(fs.readFileSync(p, "utf8"));
  if (j.private) continue;
  packages.push({ name: j.name, version: j.version, where: `packages/${d.name}` });
}

// 原生 binding 根包
const bindingPath = path.join(repoRoot, "crates/node_binding/package.json");
if (fs.existsSync(bindingPath)) {
  const j = JSON.parse(fs.readFileSync(bindingPath, "utf8"));
  packages.push({ name: j.name, version: j.version, where: "crates/node_binding" });
}

if (packages.length === 0) {
  console.error("没有找到任何可发布的包");
  process.exit(1);
}

console.log(`tag: ${rawTag}  → 期望版本 ${tagVersion}\n`);

const mismatched = packages.filter((p) => p.version !== tagVersion);
for (const p of packages) {
  const mark = p.version === tagVersion ? "OK  " : "MISMATCH";
  console.log(`  ${mark.padEnd(9)} ${p.version.padEnd(10)} ${p.name}  (${p.where})`);
}

console.log("");

if (mismatched.length > 0) {
  console.error(`[FAIL] ${mismatched.length} 个包的版本与 tag 不一致，拒绝发布`);
  console.error("");
  console.error("这两个值必须相等，因为 `napi prepublish -t npm` 会用");
  console.error("`v<package.json version>` 作为 GitHub Release 的 tag 名。");
  console.error("修法：确认 `pnpm changeset version` 已执行并提交，再用对应版本打 tag。");
  process.exit(1);
}

console.log(`[OK] ${packages.length} 个包版本一致 = ${tagVersion}，可以发布`);
