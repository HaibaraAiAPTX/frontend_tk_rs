#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "../..");

const args = process.argv.slice(2);

function printHelp() {
  console.log(`Publish aptx-ft npm packages (binding -> cli)

Usage:
  node scripts/release/publish-cli-deps.mjs [options]

Options:
  --tag <tag>            npm dist-tag, default: latest
  --access <mode>        npm publish access, default: public
  --registry <url>       custom npm registry
  --otp <code>           npm otp code
  --dry-run              print commands only, do not execute
  --skip-build           skip build steps
  --allow-dirty          allow dirty git working tree
  -h, --help             show this help
`);
}

function parseArgs(argv) {
  const options = {
    tag: "latest",
    access: "public",
    registry: "",
    otp: "",
    dryRun: false,
    skipBuild: false,
    allowDirty: false,
    help: false,
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--") {
      continue;
    }
    if (arg === "--tag") {
      options.tag = argv[++i] || "";
    } else if (arg === "--access") {
      options.access = argv[++i] || "";
    } else if (arg === "--registry") {
      options.registry = argv[++i] || "";
    } else if (arg === "--otp") {
      options.otp = argv[++i] || "";
    } else if (arg === "--dry-run") {
      options.dryRun = true;
    } else if (arg === "--skip-build") {
      options.skipBuild = true;
    } else if (arg === "--allow-dirty") {
      options.allowDirty = true;
    } else if (arg === "-h" || arg === "--help") {
      options.help = true;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!options.tag) {
    throw new Error("--tag requires a non-empty value");
  }
  if (!options.access) {
    throw new Error("--access requires a non-empty value");
  }

  return options;
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function runCommand(cmd, cmdArgs, opts = {}) {
  const cwd = opts.cwd || repoRoot;
  const printable = [cmd, ...cmdArgs].join(" ");
  console.log(`\n$ ${printable}`);

  if (opts.dryRun) {
    return;
  }

  const result = spawnSync(cmd, cmdArgs, {
    cwd,
    stdio: "inherit",
    shell: process.platform === "win32",
    env: process.env,
  });

  if (result.status !== 0) {
    throw new Error(`Command failed (${result.status}): ${printable}`);
  }
}

function ensureCleanGit(allowDirty) {
  if (allowDirty) {
    return;
  }

  const result = spawnSync("git", ["status", "--porcelain"], {
    cwd: repoRoot,
    stdio: ["ignore", "pipe", "pipe"],
    shell: process.platform === "win32",
    env: process.env,
    encoding: "utf8",
  });

  if (result.status !== 0) {
    throw new Error("Failed to run git status. Use --allow-dirty to skip this check.");
  }

  if ((result.stdout || "").trim().length > 0) {
    throw new Error("Git working tree is not clean. Commit/stash changes or pass --allow-dirty.");
  }
}

function validateVersions() {
  const bindingPkgPath = path.join(repoRoot, "crates/node_binding/package.json");
  const cliPkgPath = path.join(repoRoot, "packages/frontend-tk-cli/package.json");
  const npmDir = path.join(repoRoot, "crates/node_binding/npm");

  const bindingPkg = readJson(bindingPkgPath);
  const cliPkg = readJson(cliPkgPath);

  const versions = [
    [bindingPkg.name, bindingPkg.version, bindingPkgPath],
    [cliPkg.name, cliPkg.version, cliPkgPath],
  ];

  for (const [name, version, file] of versions) {
    if (!version || typeof version !== "string") {
      throw new Error(`Invalid version for ${name} in ${file}`);
    }
    if (version === "0.0.0") {
      throw new Error(`Version for ${name} is 0.0.0 in ${file}. Bump version before publish.`);
    }
  }

  const bindingVersion = bindingPkg.version;
  const npmPackageDirs = fs
    .readdirSync(npmDir, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join(npmDir, entry.name, "package.json"))
    .filter((file) => fs.existsSync(file));

  for (const file of npmPackageDirs) {
    const pkg = readJson(file);
    if (pkg.version !== bindingVersion) {
      throw new Error(
        `Binding version mismatch: ${file} is ${pkg.version}, expected ${bindingVersion}.`,
      );
    }
  }

  console.log("Version check passed:");
  console.log(`- ${bindingPkg.name}@${bindingPkg.version}`);
  console.log(`- ${cliPkg.name}@${cliPkg.version}`);
  console.log(`- ${npmPackageDirs.length} binding platform packages aligned`);
}

function buildPublishArgs(options) {
  const publishArgs = ["publish", "--access", options.access, "--tag", options.tag];
  if (options.registry) {
    publishArgs.push("--registry", options.registry);
  }
  if (options.otp) {
    publishArgs.push("--otp", options.otp);
  }
  if (options.dryRun) {
    publishArgs.push("--dry-run");
  }
  return publishArgs;
}

function main() {
  const options = parseArgs(args);
  if (options.help) {
    printHelp();
    return;
  }

  console.log("Preparing npm release for CLI and dependencies...");
  ensureCleanGit(options.allowDirty);
  validateVersions();

  if (!options.skipBuild) {
    runCommand("pnpm", ["--filter", "@aptx/frontend-tk-binding", "build"], { dryRun: options.dryRun });
    runCommand("pnpm", ["--filter", "@aptx/frontend-tk-cli", "build"], { dryRun: options.dryRun });
  }

  const publishArgs = buildPublishArgs(options);

  runCommand("pnpm", ["--filter", "@aptx/frontend-tk-binding", ...publishArgs], { dryRun: options.dryRun });
  runCommand("pnpm", ["--filter", "@aptx/frontend-tk-cli", ...publishArgs], { dryRun: options.dryRun });

  console.log("\nRelease flow finished.");
}

try {
  main();
} catch (error) {
  const message = error instanceof Error ? error.message : String(error);
  console.error(`\nError: ${message}`);
  process.exit(1);
}
