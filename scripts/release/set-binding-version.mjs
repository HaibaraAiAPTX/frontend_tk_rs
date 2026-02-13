#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "../..");

const bindingPkgPath = path.join(repoRoot, "crates/node_binding/package.json");
const bindingNpmDir = path.join(repoRoot, "crates/node_binding/npm");

function printHelp() {
  console.log(`Set @aptx/frontend-tk-binding version and sync all platform packages

Usage:
  node scripts/release/set-binding-version.mjs <version> [options]

Options:
  --dry-run      print changes only
  --check        only check whether versions are aligned
  -h, --help     show help
`);
}

function isSemver(value) {
  return /^\d+\.\d+\.\d+(?:-[0-9A-Za-z-.]+)?(?:\+[0-9A-Za-z-.]+)?$/.test(value);
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function writeJson(filePath, content) {
  fs.writeFileSync(filePath, `${JSON.stringify(content, null, 2)}\n`, "utf8");
}

function getPlatformPackageJsonPaths() {
  return fs
    .readdirSync(bindingNpmDir, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join(bindingNpmDir, entry.name, "package.json"))
    .filter((filePath) => fs.existsSync(filePath));
}

function parseArgs(argv) {
  const options = {
    version: "",
    dryRun: false,
    check: false,
    help: false,
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--dry-run") {
      options.dryRun = true;
    } else if (arg === "--check") {
      options.check = true;
    } else if (arg === "-h" || arg === "--help") {
      options.help = true;
    } else if (!options.version) {
      options.version = arg;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  return options;
}

function checkAligned(expectedVersion) {
  const bindingPkg = readJson(bindingPkgPath);
  const expected = expectedVersion || bindingPkg.version;
  const files = [bindingPkgPath, ...getPlatformPackageJsonPaths()];
  const mismatched = [];

  for (const file of files) {
    const pkg = readJson(file);
    if (pkg.version !== expected) {
      mismatched.push({ file, version: pkg.version });
    }
  }

  if (mismatched.length > 0) {
    console.error(`Found ${mismatched.length} version mismatch(es), expected ${expected}:`);
    for (const item of mismatched) {
      console.error(`- ${item.file}: ${item.version}`);
    }
    return false;
  }

  console.log(`All binding package versions are aligned at ${expected}.`);
  return true;
}

function setVersion(version, dryRun) {
  if (!isSemver(version)) {
    throw new Error(`Invalid semver version: ${version}`);
  }

  const files = [bindingPkgPath, ...getPlatformPackageJsonPaths()];
  let changed = 0;

  for (const file of files) {
    const pkg = readJson(file);
    const before = pkg.version;
    if (before === version) {
      continue;
    }

    pkg.version = version;
    changed += 1;

    if (dryRun) {
      console.log(`[dry-run] ${file}: ${before} -> ${version}`);
    } else {
      writeJson(file, pkg);
      console.log(`${file}: ${before} -> ${version}`);
    }
  }

  if (changed === 0) {
    console.log(`No changes. All package versions already at ${version}.`);
  } else if (dryRun) {
    console.log(`[dry-run] ${changed} file(s) would be updated.`);
  } else {
    console.log(`${changed} file(s) updated.`);
  }
}

function main() {
  const options = parseArgs(process.argv.slice(2));

  if (options.help) {
    printHelp();
    return;
  }

  if (options.check) {
    const ok = checkAligned(options.version);
    if (!ok) {
      process.exit(1);
    }
    return;
  }

  if (!options.version) {
    throw new Error("Missing <version>. Example: node .../set-binding-version.mjs 0.1.0");
  }

  setVersion(options.version, options.dryRun);
}

try {
  main();
} catch (error) {
  const message = error instanceof Error ? error.message : String(error);
  console.error(`Error: ${message}`);
  process.exit(1);
}
