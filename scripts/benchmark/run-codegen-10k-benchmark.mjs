#!/usr/bin/env node
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

function parseArgs(argv) {
  const args = {
    endpoints: 10000,
    terminals: ["axios-ts"],
    outputRoot: "target/benchmarks/codegen-10k-output",
    samplePath: "target/benchmarks/openapi-10000.json",
    reportDir: "target/benchmarks/reports",
    docPath: "docs/benchmarks/10k-endpoint-benchmark.md",
  };
  for (let i = 0; i < argv.length; i += 1) {
    const token = argv[i];
    if (token === "--endpoints") {
      args.endpoints = Number(argv[i + 1]);
      i += 1;
      continue;
    }
    if (token === "--terminal") {
      args.terminals = [argv[i + 1]];
      i += 1;
      continue;
    }
    if (token === "--output-root") {
      args.outputRoot = argv[i + 1];
      i += 1;
      continue;
    }
    if (token === "--sample") {
      args.samplePath = argv[i + 1];
      i += 1;
      continue;
    }
  }
  if (!Number.isInteger(args.endpoints) || args.endpoints <= 0) {
    throw new Error("`--endpoints` must be a positive integer.");
  }
  return args;
}

function normalizePathForTs(value) {
  return value.replace(/\\/g, "/");
}

function runCommand(cwd, cmd, argv) {
  const result = spawnSync(cmd, argv, {
    cwd,
    encoding: "utf-8",
  });
  if (result.status !== 0) {
    throw new Error(
      `command failed: ${cmd} ${argv.join(" ")}\nstdout:\n${result.stdout}\nstderr:\n${result.stderr}`,
    );
  }
  return {
    stdout: result.stdout || "",
    stderr: result.stderr || "",
  };
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf-8"));
}

function formatRatio(a, b) {
  if (!a || !b) {
    return "n/a";
  }
  return `${(a / b).toFixed(4)}x`;
}

function formatPercent(a, b) {
  if (!a || !b) {
    return "n/a";
  }
  return `${((a / b) * 100).toFixed(2)}%`;
}

function buildMarkdown(summary) {
  return `# 10k Endpoint Benchmark

- Generated at: ${summary.generatedAt}
- Machine: ${summary.machine.platform} ${summary.machine.release}
- CPU: ${summary.machine.cpuModel}
- Node: ${summary.machine.node}
- Endpoint count: ${summary.endpointCount}
- Terminals: ${summary.terminals.join(", ")}

## Result

| Scenario | Duration(ms) | Planned | Written | Skipped | CacheHit |
|---|---:|---:|---:|---:|---|
| cold-run | ${summary.cold.durationMs} | ${summary.cold.totals.plannedFiles} | ${summary.cold.totals.writtenFiles} | ${summary.cold.totals.skippedFiles} | ${summary.cold.cacheHit} |
| warm-run | ${summary.warm.durationMs} | ${summary.warm.totals.plannedFiles} | ${summary.warm.totals.writtenFiles} | ${summary.warm.totals.skippedFiles} | ${summary.warm.cacheHit} |
| dry-run | ${summary.dry.durationMs} | ${summary.dry.totals.plannedFiles} | ${summary.dry.totals.writtenFiles} | ${summary.dry.totals.skippedFiles} | ${summary.dry.cacheHit} |

## Key Metrics

- warm/cold ratio: ${formatRatio(summary.warm.durationMs, summary.cold.durationMs)}
- warm/cold percent: ${formatPercent(summary.warm.durationMs, summary.cold.durationMs)}

## Artifacts

- sample: \`${summary.paths.sample}\`
- config: \`${summary.paths.config}\`
- cold report: \`${summary.paths.coldReport}\`
- warm report: \`${summary.paths.warmReport}\`
- dry report: \`${summary.paths.dryReport}\`
`;
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..", "..");

  const samplePath = path.resolve(root, args.samplePath);
  const outputRoot = path.resolve(root, args.outputRoot);
  const reportDir = path.resolve(root, args.reportDir);
  const docPath = path.resolve(root, args.docPath);
  const configPath = path.resolve(reportDir, "benchmark.config.ts");
  const coldReport = path.resolve(reportDir, "cold-run.report.json");
  const warmReport = path.resolve(reportDir, "warm-run.report.json");
  const dryReport = path.resolve(reportDir, "dry-run.report.json");

  fs.mkdirSync(reportDir, { recursive: true });
  runCommand(root, process.execPath, [
    path.resolve(scriptDir, "generate-openapi-sample.mjs"),
    "--endpoints",
    String(args.endpoints),
    "--output",
    samplePath,
  ]);

  if (fs.existsSync(outputRoot)) {
    fs.rmSync(outputRoot, { recursive: true, force: true });
  }

  const configText = `export default {
  input: '${normalizePathForTs(samplePath)}',
  codegen: {
    outputRoot: '${normalizePathForTs(outputRoot)}',
    terminals: [${args.terminals.map((v) => `'${v}'`).join(", ")}]
  },
  performance: {
    cache: true,
    concurrency: 'auto'
  }
}
`;
  fs.writeFileSync(configPath, configText);

  const cliPath = path.resolve(root, "packages/frontend-tk-cli/bin/aptx.js");

  const cold = runCommand(root, process.execPath, [
    cliPath,
    "codegen",
    "run",
    "--profile",
    "--report-json",
    coldReport,
    "-c",
    configPath,
  ]);

  const warm = runCommand(root, process.execPath, [
    cliPath,
    "codegen",
    "run",
    "--profile",
    "--report-json",
    warmReport,
    "-c",
    configPath,
  ]);

  const dry = runCommand(root, process.execPath, [
    cliPath,
    "codegen",
    "run",
    "--dry-run",
    "--profile",
    "--report-json",
    dryReport,
    "-c",
    configPath,
  ]);

  fs.writeFileSync(path.resolve(reportDir, "cold-run.log.txt"), cold.stdout + cold.stderr);
  fs.writeFileSync(path.resolve(reportDir, "warm-run.log.txt"), warm.stdout + warm.stderr);
  fs.writeFileSync(path.resolve(reportDir, "dry-run.log.txt"), dry.stdout + dry.stderr);

  const coldJson = readJson(coldReport);
  const warmJson = readJson(warmReport);
  const dryJson = readJson(dryReport);

  const summary = {
    generatedAt: new Date().toISOString(),
    endpointCount: args.endpoints,
    terminals: args.terminals,
    machine: {
      platform: os.platform(),
      release: os.release(),
      cpuModel: os.cpus()[0]?.model || "unknown",
      node: process.version,
    },
    cold: coldJson,
    warm: warmJson,
    dry: dryJson,
    paths: {
      sample: path.relative(root, samplePath).replace(/\\/g, "/"),
      config: path.relative(root, configPath).replace(/\\/g, "/"),
      coldReport: path.relative(root, coldReport).replace(/\\/g, "/"),
      warmReport: path.relative(root, warmReport).replace(/\\/g, "/"),
      dryReport: path.relative(root, dryReport).replace(/\\/g, "/"),
    },
  };

  fs.mkdirSync(path.dirname(docPath), { recursive: true });
  fs.writeFileSync(docPath, buildMarkdown(summary));
  fs.writeFileSync(path.resolve(reportDir, "summary.json"), JSON.stringify(summary, null, 2));

  console.log(
    JSON.stringify(
      {
        doc: path.relative(root, docPath).replace(/\\/g, "/"),
        summary: path.relative(root, path.resolve(reportDir, "summary.json")).replace(/\\/g, "/"),
        coldMs: coldJson.durationMs,
        warmMs: warmJson.durationMs,
        dryMs: dryJson.durationMs,
      },
      null,
      2,
    ),
  );
}

main();
