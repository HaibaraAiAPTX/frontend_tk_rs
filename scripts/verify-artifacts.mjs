#!/usr/bin/env node
/**
 * 发布闸门：在发布任何包之前，证明 9 个平台的二进制全部就位且版本一致。
 *
 * 为什么需要它：`napi pre-publish` 对缺失的 target 只打印 warning 然后跳过，
 * 命令本身仍然成功。也就是说少一个平台的 `.node` 文件，根包照样会发布出去，
 * 而 npm 的版本不可覆盖——发出去的残缺版本无法补救。
 * 这个脚本把"静默跳过"变成"硬失败"。
 *
 * 用法：
 *   node scripts/verify-artifacts.mjs              # 校验 crates/node_binding
 *   node scripts/verify-artifacts.mjs --verbose    # 额外打印每个 target 的字节数
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");
const bindingDir = path.join(repoRoot, "crates/node_binding");
const pkgPath = path.join(bindingDir, "package.json");

const verbose = process.argv.includes("--verbose");

/**
 * napi target triple 里的架构名与 npm 平台包目录名不同名，必须先归一化：
 *   x86_64 -> x64, aarch64 -> arm64, armv7 -> arm
 */
const ARCH_ALIAS = {
  x86_64: "x64",
  aarch64: "arm64",
  armv7: "arm",
  i686: "ia32",
};

/** 把 napi target triple 映射成 npm 平台包的目录名（与 npm/ 下的结构一致）。 */
function targetToNpmDir(target) {
  const [rawArch, ...rest] = target.split("-");
  const arch = ARCH_ALIAS[rawArch] ?? rawArch;
  const os = rest[0];
  const abi = rest[1] ?? "";

  if (os === "apple" && abi === "darwin") return `darwin-${arch}`;
  if (os === "pc" && abi === "windows") return `win32-${arch}-${rest[2] ?? "msvc"}`;
  if (os === "unknown" && abi === "linux") return `linux-${arch}-${rest[2] ?? "gnu"}`;
  throw new Error(`无法识别的 target triple: ${target}`);
}

function fail(message, details = []) {
  console.error(`\n[FAIL] ${message}`);
  for (const line of details) console.error(`  - ${line}`);
  console.error("");
  process.exit(1);
}

function main() {
  if (!fs.existsSync(pkgPath)) fail(`找不到 ${pkgPath}`);
  const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf8"));

  const targets = pkg.napi?.targets;
  if (!Array.isArray(targets) || targets.length === 0) {
    fail("package.json 里没有 napi.targets，无法确定需要哪些平台包");
  }

  const binaryName = pkg.napi?.binaryName ?? pkg.name;
  const version = pkg.version;
  // null 表示源码里还没有这个字段（首次发布前），区别于空对象（有字段但没内容）。
  const optionalDeps = pkg.optionalDependencies ?? null;

  console.log(`包: ${pkg.name}@${version}`);
  console.log(`binaryName: ${binaryName}`);
  console.log(`targets: ${targets.length} 个`);
  if (optionalDeps === null) {
    console.log("optionalDependencies: 未声明（首次发布，将由 napi prepublish 写入）");
  }
  console.log("");

  const missing = [];
  const versionMismatch = [];
  const missingDep = [];
  let totalBytes = 0;

  for (const target of targets) {
    const npmDir = targetToNpmDir(target);
    const platformPkgName = `${pkg.name}-${npmDir}`;
    const platformPkgPath = path.join(bindingDir, "npm", npmDir, "package.json");
    const nodeFile = path.join(bindingDir, "npm", npmDir, `${binaryName}.${npmDir}.node`);

    // 1. 平台包目录和 package.json 必须存在
    if (!fs.existsSync(platformPkgPath)) {
      missing.push(`${npmDir}: 缺少 npm/${npmDir}/package.json`);
      continue;
    }

    // 2. 该平台的 .node 文件必须存在且非空
    if (!fs.existsSync(nodeFile)) {
      missing.push(`${npmDir}: 缺少 ${path.basename(nodeFile)}`);
      continue;
    }
    const size = fs.statSync(nodeFile).size;
    if (size === 0) {
      missing.push(`${npmDir}: ${path.basename(nodeFile)} 是 0 字节`);
      continue;
    }
    totalBytes += size;

    // 3. 平台包自身的 version 必须与根包一致
    const platformPkg = JSON.parse(fs.readFileSync(platformPkgPath, "utf8"));
    if (platformPkg.version !== version) {
      versionMismatch.push(`${npmDir}: 平台包 version=${platformPkg.version}，根包 version=${version}`);
    }

    // 4. 根包的 optionalDependencies 必须精确指向这个版本。
    //    首次发布前源码里没有这个字段（由 napi prepublish 在发布时写入），
    //    此时跳过该项校验——第 1、2 步已经证明二进制和平台包就位。
    if (optionalDeps !== null) {
      const declared = optionalDeps[platformPkgName];
      if (declared === undefined) {
        missingDep.push(`${npmDir}: 根包 optionalDependencies 缺少 ${platformPkgName}`);
      } else if (declared !== version) {
        versionMismatch.push(`${npmDir}: optionalDependencies 写的是 ${declared}，期望 ${version}`);
      }
    }

    if (verbose) {
      console.log(`  OK  ${npmDir.padEnd(26)} ${(size / 1024 / 1024).toFixed(2)} MB`);
    }
  }

  // 5. optionalDependencies 里不能有 napi.targets 之外的陈旧条目
  const staleDeps = (optionalDeps === null ? [] : Object.keys(optionalDeps)).filter((depName) => {
    const suffix = depName.slice(`${pkg.name}-`.length);
    return !targets.some((t) => targetToNpmDir(t) === suffix);
  });

  if (!verbose) {
    console.log(`  ${targets.length} 个 target 检查完毕`);
  }
  console.log(`  二进制合计: ${(totalBytes / 1024 / 1024).toFixed(2)} MB\n`);

  if (missing.length) {
    fail(`${missing.length} 个 target 的二进制缺失或无效，拒绝发布`, missing);
  }
  if (versionMismatch.length) {
    fail(`${versionMismatch.length} 处版本不一致，拒绝发布`, versionMismatch);
  }
  if (missingDep.length) {
    fail(`${missingDep.length} 个平台包未被根包声明，客户端无法解析到它们`, missingDep);
  }
  if (staleDeps.length) {
    fail(`根包 optionalDependencies 有 ${staleDeps.length} 个陈旧条目（napi.targets 里已不存在）`, staleDeps);
  }

  console.log(`[OK] ${targets.length} 个平台全部就位，版本一致，可以发布`);
}

main();
