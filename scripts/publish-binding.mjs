#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "..");
const bindingDir = path.join(repoRoot, "crates/node_binding");

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

  const result = spawnSync(cmd, cmdArgs, {
    cwd,
    stdio: "inherit",
    shell: process.platform === "win32",
    env: process.env,
  });

  if (result.status !== 0) {
    throw new Error(`Command failed: ${printable}`);
  }
}

function getCurrentRegistry() {
  const result = spawnSync("npm", ["config", "get", "registry"], {
    encoding: "utf8",
    shell: process.platform === "win32",
  });
  return result.stdout.trim();
}

function setRegistry(registry) {
  runCommand("npm", ["config", "set", "registry", registry]);
}

function main() {
  const options = parseArgs();
  if (options.help) {
    printHelp();
    return;
  }

  console.log("=== Download Release from GitHub ===");
  
  const tagArg = options.tag ? [options.tag] : [];
  runCommand("gh", ["release", "download", ...tagArg, "--dir", bindingDir, "--pattern", "*.node"]);

  console.log("\n=== Generate npm packages ===");
  runCommand("pnpm", ["run", "artifacts"]);

  console.log("\n=== Publish to private registry ===");
  
  const originalRegistry = getCurrentRegistry();
  console.log(`Current registry: ${originalRegistry}`);
  
  setRegistry(PRIVATE_REGISTRY);
  console.log(`Switched to: ${PRIVATE_REGISTRY}`);

  try {
    runCommand("npx", ["napi", "pre-publish", "--no-gh-release"]);
  } finally {
    setRegistry(originalRegistry);
    console.log(`Restored registry: ${originalRegistry}`);
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
