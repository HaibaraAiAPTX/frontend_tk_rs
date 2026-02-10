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
    runs: 1,
    warmup: 0,
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
    if (token === "--runs") {
      args.runs = Number(argv[i + 1]);
      i += 1;
      continue;
    }
    if (token === "--warmup") {
      args.warmup = Number(argv[i + 1]);
      i += 1;
    }
  }

  if (!Number.isInteger(args.endpoints) || args.endpoints <= 0) {
    throw new Error("`--endpoints` must be a positive integer.");
  }
  if (!Number.isInteger(args.runs) || args.runs <= 0) {
    throw new Error("`--runs` must be a positive integer.");
  }
  if (!Number.isInteger(args.warmup) || args.warmup < 0) {
    throw new Error("`--warmup` must be a non-negative integer.");
  }

  return args;
}

function normalizePathForTs(value) {
  return value.replace(/\\/g, "/");
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf-8"));
}

function runOrThrow(cmd, argv, options = {}) {
  const result = spawnSync(cmd, argv, {
    encoding: "utf-8",
    ...options,
  });
  if (result.status !== 0) {
    throw new Error(
      `command failed: ${cmd} ${argv.join(" ")}\nstdout:\n${result.stdout || ""}\nstderr:\n${result.stderr || ""}`,
    );
  }
  return {
    stdout: result.stdout || "",
    stderr: result.stderr || "",
  };
}

function commandExists(cmd) {
  const result = spawnSync(cmd, ["--version"], {
    encoding: "utf-8",
  });
  return result.status === 0;
}

function quoteForShell(value) {
  if (process.platform === "win32") {
    return `"${value.replace(/"/g, '""')}"`;
  }
  return `'${value.replace(/'/g, `'"'"'`)}'`;
}

function buildNodeCommand(filePath, args) {
  return [quoteForShell(process.execPath), quoteForShell(filePath), ...args.map(quoteForShell)].join(" ");
}

function runHyperfine(cwd, options) {
  const argv = [
    "--runs",
    String(options.runs),
    "--warmup",
    String(options.warmup),
    "--command-name",
    options.commandName,
    "--export-json",
    options.exportJson,
    options.command,
  ];

  const output = runOrThrow("hyperfine", argv, { cwd });
  fs.writeFileSync(options.logPath, output.stdout + output.stderr);
}

function readHyperfineSummary(filePath) {
  const json = readJson(filePath);
  const first = json.results && json.results[0] ? json.results[0] : undefined;
  if (!first) {
    return {
      meanMs: null,
      stddevMs: null,
      minMs: null,
      maxMs: null,
    };
  }
  return {
    meanMs: Number((first.mean * 1000).toFixed(3)),
    stddevMs: Number((first.stddev * 1000).toFixed(3)),
    minMs: Number((first.min * 1000).toFixed(3)),
    maxMs: Number((first.max * 1000).toFixed(3)),
  };
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

function formatMs(v) {
  if (v === null || v === undefined || Number.isNaN(v)) {
    return "n/a";
  }
  return String(v);
}

function buildMarkdown(summary) {
  return `# 10k Endpoint Benchmark

- Generated at: ${summary.generatedAt}
- Machine: ${summary.machine.platform} ${summary.machine.release}
- CPU: ${summary.machine.cpuModel}
- Node: ${summary.machine.node}
- Endpoint count: ${summary.endpointCount}
- Terminals: ${summary.terminals.join(", ")}
- Framework: hyperfine (${summary.hyperfine.version})
- Hyperfine runs: ${summary.hyperfine.runs}
- Hyperfine warmup: ${summary.hyperfine.warmup}

## Result

| Scenario | Duration(ms, app report) | Hyperfine mean(ms) | Hyperfine stddev(ms) | Planned | Written | Skipped | CacheHit |
|---|---:|---:|---:|---:|---:|---:|---|
| cold-run | ${summary.cold.durationMs} | ${formatMs(summary.cold.hyperfine.meanMs)} | ${formatMs(summary.cold.hyperfine.stddevMs)} | ${summary.cold.totals.plannedFiles} | ${summary.cold.totals.writtenFiles} | ${summary.cold.totals.skippedFiles} | ${summary.cold.cacheHit} |
| warm-run | ${summary.warm.durationMs} | ${formatMs(summary.warm.hyperfine.meanMs)} | ${formatMs(summary.warm.hyperfine.stddevMs)} | ${summary.warm.totals.plannedFiles} | ${summary.warm.totals.writtenFiles} | ${summary.warm.totals.skippedFiles} | ${summary.warm.cacheHit} |
| dry-run | ${summary.dry.durationMs} | ${formatMs(summary.dry.hyperfine.meanMs)} | ${formatMs(summary.dry.hyperfine.stddevMs)} | ${summary.dry.totals.plannedFiles} | ${summary.dry.totals.writtenFiles} | ${summary.dry.totals.skippedFiles} | ${summary.dry.cacheHit} |

## Key Metrics

- warm/cold ratio (app report): ${formatRatio(summary.warm.durationMs, summary.cold.durationMs)}
- warm/cold percent (app report): ${formatPercent(summary.warm.durationMs, summary.cold.durationMs)}
- warm/cold ratio (hyperfine mean): ${formatRatio(summary.warm.hyperfine.meanMs, summary.cold.hyperfine.meanMs)}
- warm/cold percent (hyperfine mean): ${formatPercent(summary.warm.hyperfine.meanMs, summary.cold.hyperfine.meanMs)}

## Artifacts

- sample: \`${summary.paths.sample}\`
- config: \`${summary.paths.config}\`
- cold report: \`${summary.paths.coldReport}\`
- warm report: \`${summary.paths.warmReport}\`
- dry report: \`${summary.paths.dryReport}\`
- cold hyperfine: \`${summary.paths.coldHyperfine}\`
- warm hyperfine: \`${summary.paths.warmHyperfine}\`
- dry hyperfine: \`${summary.paths.dryHyperfine}\`
`;
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..", "..");

  if (!commandExists("hyperfine")) {
    throw new Error(
      "`hyperfine` is required but was not found in PATH. Install it first (Windows example: `winget install sharkdp.hyperfine`).",
    );
  }

  const samplePath = path.resolve(root, args.samplePath);
  const outputRoot = path.resolve(root, args.outputRoot);
  const reportDir = path.resolve(root, args.reportDir);
  const docPath = path.resolve(root, args.docPath);
  const configPath = path.resolve(reportDir, "benchmark.config.ts");

  const coldReport = path.resolve(reportDir, "cold-run.report.json");
  const warmReport = path.resolve(reportDir, "warm-run.report.json");
  const dryReport = path.resolve(reportDir, "dry-run.report.json");

  const coldHyperfineJson = path.resolve(reportDir, "cold-run.hyperfine.json");
  const warmHyperfineJson = path.resolve(reportDir, "warm-run.hyperfine.json");
  const dryHyperfineJson = path.resolve(reportDir, "dry-run.hyperfine.json");

  const coldHyperfineLog = path.resolve(reportDir, "cold-run.hyperfine.log.txt");
  const warmHyperfineLog = path.resolve(reportDir, "warm-run.hyperfine.log.txt");
  const dryHyperfineLog = path.resolve(reportDir, "dry-run.hyperfine.log.txt");

  fs.mkdirSync(reportDir, { recursive: true });

  runOrThrow(process.execPath, [
    path.resolve(scriptDir, "generate-openapi-sample.mjs"),
    "--endpoints",
    String(args.endpoints),
    "--output",
    samplePath,
  ], { cwd: root });

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

  const coldCommand = buildNodeCommand(cliPath, [
    "codegen",
    "run",
    "--profile",
    "--report-json",
    coldReport,
    "-c",
    configPath,
  ]);

  runHyperfine(root, {
    runs: args.runs,
    warmup: args.warmup,
    commandName: "cold-run",
    exportJson: coldHyperfineJson,
    command: coldCommand,
    logPath: coldHyperfineLog,
  });

  runOrThrow(process.execPath, [
    cliPath,
    "codegen",
    "run",
    "-c",
    configPath,
  ], { cwd: root });

  const warmCommand = buildNodeCommand(cliPath, [
    "codegen",
    "run",
    "--profile",
    "--report-json",
    warmReport,
    "-c",
    configPath,
  ]);

  runHyperfine(root, {
    runs: args.runs,
    warmup: args.warmup,
    commandName: "warm-run",
    exportJson: warmHyperfineJson,
    command: warmCommand,
    logPath: warmHyperfineLog,
  });

  const dryCommand = buildNodeCommand(cliPath, [
    "codegen",
    "run",
    "--dry-run",
    "--profile",
    "--report-json",
    dryReport,
    "-c",
    configPath,
  ]);

  runHyperfine(root, {
    runs: args.runs,
    warmup: args.warmup,
    commandName: "dry-run",
    exportJson: dryHyperfineJson,
    command: dryCommand,
    logPath: dryHyperfineLog,
  });

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
    hyperfine: {
      version: runOrThrow("hyperfine", ["--version"]).stdout.trim(),
      runs: args.runs,
      warmup: args.warmup,
    },
    cold: {
      ...coldJson,
      hyperfine: readHyperfineSummary(coldHyperfineJson),
    },
    warm: {
      ...warmJson,
      hyperfine: readHyperfineSummary(warmHyperfineJson),
    },
    dry: {
      ...dryJson,
      hyperfine: readHyperfineSummary(dryHyperfineJson),
    },
    paths: {
      sample: path.relative(root, samplePath).replace(/\\/g, "/"),
      config: path.relative(root, configPath).replace(/\\/g, "/"),
      coldReport: path.relative(root, coldReport).replace(/\\/g, "/"),
      warmReport: path.relative(root, warmReport).replace(/\\/g, "/"),
      dryReport: path.relative(root, dryReport).replace(/\\/g, "/"),
      coldHyperfine: path.relative(root, coldHyperfineJson).replace(/\\/g, "/"),
      warmHyperfine: path.relative(root, warmHyperfineJson).replace(/\\/g, "/"),
      dryHyperfine: path.relative(root, dryHyperfineJson).replace(/\\/g, "/"),
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
        coldMeanMs: summary.cold.hyperfine.meanMs,
        warmMeanMs: summary.warm.hyperfine.meanMs,
        dryMeanMs: summary.dry.hyperfine.meanMs,
      },
      null,
      2,
    ),
  );
}

main();
