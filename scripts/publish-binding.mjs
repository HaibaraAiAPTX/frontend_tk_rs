#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "..");
const bindingDir = path.join(repoRoot, "crates/node_binding");
const artifactsDir = path.join(bindingDir, "artifacts");

const args = process.argv.slice(2);

const PRIVATE_REGISTRY = "https://winserver.devserver.ink:14873/";

function printHelp() {
  console.log(`Download GitHub Release and publish to private registry

Usage:
  node scripts/publish-binding.mjs [options]

Options:
  --tag <tag>         GitHub release tag, default: latest
  --yes               Skip confirmation
  -h, --help         show help
`);
}

function parseArgs() {
  const options = { tag: "", yes: false, help: false };
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--tag") {
      options.tag = args[++i] || "";
    } else if (arg === "--yes") {
      options.yes = true;
    } else if (arg === "-h" || arg === "--help") {
      options.help = true;
    }
  }
  return options;
}

function runCommand(cmd, cmdArgs, opts = {}) {
  const cwd = opts.cwd || bindingDir;
  const printable = [cmd, ...cmdArgs].join(" ");
  console.log(`\n$ ${printable}`);

  const env = { ...process.env, ...opts.env };
  const result = spawnSync(cmd, cmdArgs, {
    cwd,
    stdio: "inherit",
    shell: process.platform === "win32",
    env,
  });

  if (result.status !== 0) {
    throw new Error(`Command failed: ${printable}`);
  }
}

function ensureArtifactsDir() {
  if (!fs.existsSync(artifactsDir)) {
    fs.mkdirSync(artifactsDir, { recursive: true });
  }
}

function createNpmrc(registry) {
  const npmrcPath = path.join(bindingDir, ".npmrc");
  fs.writeFileSync(npmrcPath, `registry=${registry}\n`);
  console.log(`Created .npmrc with registry: ${registry}`);
}

function removeNpmrc() {
  const npmrcPath = path.join(bindingDir, ".npmrc");
  if (fs.existsSync(npmrcPath)) {
    fs.unlinkSync(npmrcPath);
    console.log("Removed .npmrc");
  }
}

function main() {
  const options = parseArgs();
  if (options.help) {
    printHelp();
    return;
  }

  ensureArtifactsDir();

  console.log("=== Download Release from GitHub ===");
  
  const tagArg = options.tag ? [options.tag] : [];
  runCommand("gh", ["release", "download", ...tagArg, "--dir", artifactsDir, "--pattern", "*.node"]);

  console.log("\n=== Generate npm packages ===");
  runCommand("pnpm", ["run", "artifacts"]);

  console.log("\n=== Sync version with git tag ===");
  runCommand("pnpm", ["run", "version"]);

  console.log("\n=== Publish to private registry ===");
  
  createNpmrc(PRIVATE_REGISTRY);

  // Use environment variable to force registry for npm/pnpm
  const publishEnv = {
    NPM_CONFIG_REGISTRY: PRIVATE_REGISTRY,
    PNPM_CONFIG_REGISTRY: PRIVATE_REGISTRY,
  };

  try {
    // 发布平台二进制包
    runCommand("npx", ["napi", "prepublish", "--no-gh-release"], { env: publishEnv });

    // 发布主包
    runCommand("pnpm", ["publish", "--no-git-checks"], { env: publishEnv });
  } finally {
    removeNpmrc();
  }

  console.log("\n=== Done ===");
}

try {
  main();
} catch (error) {
  const message = error instanceof Error ? error.message : String(error);
  console.error(`\nError: ${message}`);
  process.exit(1);
}
