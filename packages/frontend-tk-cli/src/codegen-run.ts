/**
 * codegen:run convenience command
 *
 * This is a CLI-specific convenience command that orchestrates multi-terminal code generation.
 * It combines the functionality of multiple plugins' renderers.
 */

import path from "path";
import fs from "fs";
import os from "os";
import crypto from "crypto";
import type {
  CommandDescriptor,
  PluginContext,
  OptionDescriptor,
} from "@aptx/frontend-tk-core";
import { runCli } from "@aptx/frontend-tk-binding";
import { getInput } from "./command/input/get-input";
import { ensureAbsolutePath } from "./utils";

// Types
type TerminalConfig = string | { id: string; output?: string };

type ClientImportConfig = {
  mode: "global" | "local" | "package";
  clientPath?: string;
  clientPackage?: string;
  importName?: string;
};

type CodegenRunOptions = {
  dryRun: boolean;
  profile: boolean;
  reportJson?: string;
  concurrencyOverride?: "auto" | number;
  cacheOverride?: boolean;
  outputRoot?: string;
  terminals?: string[];
  clientMode?: "global" | "local" | "package";
  clientPath?: string;
  clientPackage?: string;
  clientImportName?: string;
  input?: string;
};

type TerminalRunReport = {
  terminalId: string;
  channel: "native" | "script";
  output: string;
  plannedFiles: number;
  writtenFiles: number;
  skippedFiles: number;
  durationMs: number;
  dryRun: boolean;
  files: Array<{
    path: string;
    sizeBytes: number;
  }>;
  endpoints: Array<{
    operationName: string;
    method: string;
    path: string;
    matchedFile?: string;
  }>;
};

type CodegenRunReport = {
  input: string;
  dryRun: boolean;
  cacheEnabled: boolean;
  cacheHit: boolean;
  concurrency: number;
  startedAt: string;
  durationMs: number;
  terminalReports: TerminalRunReport[];
  totals: {
    plannedFiles: number;
    writtenFiles: number;
    skippedFiles: number;
  };
};

type GeneratorProjectContextIR = {
  package_name: string;
  api_base_path?: string | null;
  terminals: string[];
  retry_ownership?: string | null;
};

type GeneratorEndpointIR = {
  namespace: string[];
  operation_name: string;
  method: string;
  path: string;
  input_type_name: string;
  output_type_name: string;
  request_body_field?: string | null;
  query_fields: string[];
  path_fields: string[];
  has_request_options: boolean;
  supports_query: boolean;
  supports_mutation: boolean;
  deprecated: boolean;
};

type GeneratorInputIR = {
  project: GeneratorProjectContextIR;
  endpoints: GeneratorEndpointIR[];
};

type FileInventoryItem = {
  relativePath: string;
  absolutePath?: string;
  sizeBytes: number;
  content?: string;
};

// Supported terminals - these are built-in terminals from the Rust binding
const SUPPORTED_TERMINALS = [
  { id: "axios-ts", status: "available" },
  { id: "axios-js", status: "available" },
  { id: "uniapp", status: "available" },
  { id: "functions", status: "available" },
  { id: "react-query", status: "available" },
  { id: "vue-query", status: "available" },
];

// Utility functions
function resolveTerminalId(terminal: TerminalConfig): string {
  return typeof terminal === "string" ? terminal : terminal.id;
}

function resolveTerminalOutput(
  terminal: TerminalConfig,
  outputRoot: string,
  terminalId: string,
): string {
  if (typeof terminal === "object" && terminal.output) {
    return ensureAbsolutePath(terminal.output);
  }
  return ensureAbsolutePath(path.join(outputRoot, "services", terminalId));
}

function isBuiltInTerminal(terminalId: string): boolean {
  return SUPPORTED_TERMINALS.some((item) => item.id === terminalId);
}

function parseBooleanLike(value: string | undefined): boolean | undefined {
  if (!value) {
    return undefined;
  }
  if (value === "true" || value === "1") {
    return true;
  }
  if (value === "false" || value === "0") {
    return false;
  }
  return undefined;
}

function resolveConcurrency(value: "auto" | number | undefined): number {
  if (value === "auto" || value === undefined) {
    return Math.max(1, os.cpus().length);
  }
  return Math.max(1, value);
}

function ensureDirectoryForFile(filePath: string): void {
  const dir = path.dirname(filePath);
  fs.mkdirSync(dir, { recursive: true });
}

function hashText(text: string): string {
  return crypto.createHash("sha256").update(text).digest("hex");
}

function hashFile(filePath: string): string {
  const buf = fs.readFileSync(filePath);
  return crypto.createHash("sha256").update(buf).digest("hex");
}

function getCacheFilePath(outputRoot: string): string {
  return path.join(outputRoot, ".aptx-cache", "run-cache.json");
}

function readCacheState(cacheFile: string): {
  key?: string;
  report?: CodegenRunReport;
} {
  if (!fs.existsSync(cacheFile)) {
    return {};
  }
  try {
    const raw = fs.readFileSync(cacheFile, "utf-8");
    return JSON.parse(raw) as { key?: string; report?: CodegenRunReport };
  } catch {
    return {};
  }
}

function writeCacheState(
  cacheFile: string,
  key: string,
  report?: CodegenRunReport,
): void {
  ensureDirectoryForFile(cacheFile);
  fs.writeFileSync(
    cacheFile,
    JSON.stringify(
      {
        key,
        updatedAt: new Date().toISOString(),
        report,
      },
      null,
      2,
    ),
  );
}

function listFilesRecursive(root: string): string[] {
  if (!fs.existsSync(root)) {
    return [];
  }
  const result: string[] = [];
  const walk = (current: string) => {
    const entries = fs.readdirSync(current, { withFileTypes: true });
    entries.forEach((entry) => {
      const full = path.join(current, entry.name);
      if (entry.isDirectory()) {
        walk(full);
      } else {
        result.push(full);
      }
    });
  };
  walk(root);
  return result;
}

function toPascalCase(name: string): string {
  return name ? `${name[0].toUpperCase()}${name.slice(1)}` : name;
}

function buildFileInventoryFromDirectory(root: string): FileInventoryItem[] {
  const files = listFilesRecursive(root);
  return files.map((absolutePath) => {
    const stat = fs.statSync(absolutePath);
    const relativePath = path.relative(root, absolutePath).replace(/\\/g, "/");
    return {
      relativePath,
      absolutePath,
      sizeBytes: stat.size,
      content: fs.readFileSync(absolutePath, "utf-8"),
    };
  });
}

function buildEndpointMappings(
  ir: GeneratorInputIR,
  files: FileInventoryItem[],
): Array<{
  operationName: string;
  method: string;
  path: string;
  matchedFile?: string;
}> {
  return ir.endpoints.map((endpoint) => {
    const pascalName = toPascalCase(endpoint.operation_name);
    const matched = files.find((file) => {
      const text = file.content || "";
      return text.includes(endpoint.operation_name) || text.includes(pascalName);
    });
    return {
      operationName: endpoint.operation_name,
      method: endpoint.method,
      path: endpoint.path,
      matchedFile: matched?.relativePath,
    };
  });
}

function buildTerminalReport(
  base: Omit<TerminalRunReport, "files" | "endpoints">,
  files: FileInventoryItem[],
  ir?: GeneratorInputIR,
): TerminalRunReport {
  return {
    ...base,
    files: files.map((item) => ({
      path: item.relativePath,
      sizeBytes: item.sizeBytes,
    })),
    endpoints: ir ? buildEndpointMappings(ir, files) : [],
  };
}

function buildCodegenCacheKey(inputPath: string, payload: object): string {
  const inputHash = hashFile(inputPath);
  return hashText(`${inputHash}:${JSON.stringify(payload)}`);
}

function buildIrSnapshotByNativeCommand(input: string): GeneratorInputIR {
  const tempPath = ensureAbsolutePath(
    path.join(os.tmpdir(), `aptx-ir-${Date.now()}-${Math.random().toString(36).slice(2)}.json`),
  );
  try {
    runCli({
      input,
      command: "ir:snapshot",
      options: ["--output", tempPath],
      plugin: [],
    });
    const text = fs.readFileSync(tempPath, "utf-8");
    return JSON.parse(text) as GeneratorInputIR;
  } finally {
    if (fs.existsSync(tempPath)) {
      fs.rmSync(tempPath, { force: true });
    }
  }
}

function runBuiltInTerminalCodegen(
  terminalId: string,
  input: string,
  output: string,
  clientImport?: ClientImportConfig,
): void {
  const options = ["--terminal", terminalId, "--output", output];

  if (clientImport) {
    if (clientImport.mode && clientImport.mode !== "global") {
      options.push("--client-mode", clientImport.mode);
    }
    if (clientImport.mode === "local" && clientImport.clientPath) {
      options.push("--client-path", clientImport.clientPath);
    }
    if (clientImport.mode === "package" && clientImport.clientPackage) {
      options.push("--client-package", clientImport.clientPackage);
    }
    if (clientImport.importName) {
      options.push("--client-import-name", clientImport.importName);
    }
  }

  runCli({
    input,
    command: "terminal:codegen",
    options,
    plugin: [],
  });
}

async function runWithConcurrency<T>(
  tasks: Array<() => Promise<T>>,
  concurrency: number,
): Promise<T[]> {
  if (!tasks.length) {
    return [];
  }
  const limit = Math.max(1, concurrency);
  const results: T[] = new Array(tasks.length);
  let cursor = 0;
  const workers = Array.from(
    { length: Math.min(limit, tasks.length) },
    async () => {
      while (true) {
        const current = cursor;
        cursor += 1;
        if (current >= tasks.length) {
          break;
        }
        results[current] = await tasks[current]();
      }
    },
  );
  await Promise.all(workers);
  return results;
}

// Main codegen run handler
async function runCodegen(options: CodegenRunOptions): Promise<void> {
  if (!options.input) {
    throw new Error("`input` is required. Use -i or set config.input.");
  }

  const outputRoot = ensureAbsolutePath(options.outputRoot || "./generated");

  // Convert CLI terminal strings to TerminalConfig format
  const cliTerminals: TerminalConfig[] = (options.terminals || []).map((id) => ({
    id,
  }));
  if (!cliTerminals.length) {
    throw new Error(
      "`--terminal` parameter is required (e.g., --terminal axios-ts). Use --terminal multiple times for multiple terminals.",
    );
  }

  // Build client import config from CLI
  const clientImport: ClientImportConfig | undefined = options.clientMode
    ? {
        mode: options.clientMode,
        clientPath: options.clientPath,
        clientPackage: options.clientPackage,
        importName: options.clientImportName,
      }
    : undefined;

  const resolvedInput = await getInput(options.input);
  const irSnapshotCache = buildIrSnapshotByNativeCommand(resolvedInput);

  const terminalReports: TerminalRunReport[] = [];
  const builtInTerminalTasks: Array<{
    terminal: TerminalConfig;
    terminalId: string;
    output: string;
  }> = [];

  for (const terminal of cliTerminals) {
    const terminalId = resolveTerminalId(terminal);
    const output = resolveTerminalOutput(terminal, outputRoot, terminalId);
    if (isBuiltInTerminal(terminalId)) {
      builtInTerminalTasks.push({ terminal, terminalId, output });
    } else {
      throw new Error(
        `Terminal \`${terminalId}\` is not supported. Supported terminals: ${SUPPORTED_TERMINALS.map((t) => t.id).join(", ")}`,
      );
    }
  }

  const concurrency = resolveConcurrency(options.concurrencyOverride);
  const cacheEnabled = options.cacheOverride ?? false;
  const cachePayload = {
    terminals: cliTerminals,
    runOptions: {
      dryRun: options.dryRun,
      concurrency,
    },
  };
  const cacheFile = getCacheFilePath(outputRoot);
  const cacheState = readCacheState(cacheFile);
  const cacheKey = buildCodegenCacheKey(resolvedInput, cachePayload);
  const requiredOutputs = builtInTerminalTasks.map((item) => item.output);
  const outputsReady = requiredOutputs.every((item) => fs.existsSync(item));
  const cacheHit =
    cacheEnabled && !options.dryRun && outputsReady && cacheState.key === cacheKey;

  const startedAt = new Date();
  const runStartAt = Date.now();

  if (cacheHit) {
    const report: CodegenRunReport = cacheState.report
      ? {
          ...cacheState.report,
          input: resolvedInput,
          dryRun: false,
          cacheEnabled: true,
          cacheHit: true,
          concurrency,
          startedAt: startedAt.toISOString(),
          durationMs: Date.now() - runStartAt,
        }
      : {
          input: resolvedInput,
          dryRun: false,
          cacheEnabled: true,
          cacheHit: true,
          concurrency,
          startedAt: startedAt.toISOString(),
          durationMs: Date.now() - runStartAt,
          terminalReports: [],
          totals: {
            plannedFiles: 0,
            writtenFiles: 0,
            skippedFiles: 0,
          },
        };
    if (options.profile) {
      console.log(
        `[profile] codegen cache hit, skipped generation, duration=${report.durationMs}ms`,
      );
    }
    if (options.reportJson) {
      const reportFile = ensureAbsolutePath(options.reportJson);
      ensureDirectoryForFile(reportFile);
      fs.writeFileSync(reportFile, JSON.stringify(report, null, 2));
      console.log(`report written: ${reportFile}`);
    }
    return;
  }

  if (builtInTerminalTasks.length) {
    const builtInRunTasks = builtInTerminalTasks.map((task) => async () => {
      const outputTarget = options.dryRun
        ? ensureAbsolutePath(
            path.join(
              os.tmpdir(),
              `aptx-ir-${task.terminalId}-${Date.now()}`,
            ),
          )
        : task.output;
      const started = Date.now();
      runBuiltInTerminalCodegen(
        task.terminalId,
        resolvedInput,
        outputTarget,
        clientImport,
      );
      const outputInventory = buildFileInventoryFromDirectory(outputTarget);
      const reportItem = buildTerminalReport(
        {
          terminalId: task.terminalId,
          channel: "native",
          output: outputTarget,
          plannedFiles: outputInventory.length,
          writtenFiles: options.dryRun ? 0 : outputInventory.length,
          skippedFiles: 0,
          durationMs: Date.now() - started,
          dryRun: options.dryRun,
        },
        outputInventory,
        irSnapshotCache,
      );
      if (options.dryRun && fs.existsSync(outputTarget)) {
        fs.rmSync(outputTarget, { recursive: true, force: true });
      }
      return reportItem;
    });
    const builtInReports = await runWithConcurrency(builtInRunTasks, concurrency);
    terminalReports.push(...builtInReports);
  }

  const totals = terminalReports.reduce(
    (acc, item) => {
      acc.plannedFiles += item.plannedFiles;
      acc.writtenFiles += item.writtenFiles;
      acc.skippedFiles += item.skippedFiles;
      return acc;
    },
    {
      plannedFiles: 0,
      writtenFiles: 0,
      skippedFiles: 0,
    },
  );

  const report: CodegenRunReport = {
    input: resolvedInput,
    dryRun: options.dryRun,
    cacheEnabled,
    cacheHit: false,
    concurrency,
    startedAt: startedAt.toISOString(),
    durationMs: Date.now() - runStartAt,
    terminalReports,
    totals,
  };

  if (options.profile) {
    console.log(
      `[profile] codegen completed: duration=${report.durationMs}ms planned=${totals.plannedFiles} written=${totals.writtenFiles} skipped=${totals.skippedFiles}`,
    );
    terminalReports.forEach((item) => {
      console.log(
        `[profile] ${item.channel}:${item.terminalId} duration=${item.durationMs}ms planned=${item.plannedFiles} written=${item.writtenFiles} skipped=${item.skippedFiles}`,
      );
    });
  }

  if (options.reportJson) {
    const reportFile = ensureAbsolutePath(options.reportJson);
    ensureDirectoryForFile(reportFile);
    fs.writeFileSync(reportFile, JSON.stringify(report, null, 2));
    console.log(`report written: ${reportFile}`);
  }

  if (cacheEnabled && !options.dryRun) {
    writeCacheState(cacheFile, cacheKey, report);
  }
}

// Command descriptor
export function createCodegenRunCommand(): CommandDescriptor {
  const options: OptionDescriptor[] = [
    {
      flags: "-i, --input <path>",
      description: "Input OpenAPI path/url",
      required: true,
    },
    {
      flags: "--dry-run",
      description: "Build plan without writing files",
      defaultValue: false,
    },
    {
      flags: "--profile",
      description: "Print execution timing summary",
      defaultValue: false,
    },
    {
      flags: "--report-json <file>",
      description: "Write execution report JSON",
    },
    {
      flags: "--concurrency <value>",
      description: "Override concurrency, e.g. auto/4",
    },
    {
      flags: "--cache <true|false>",
      description: "Override incremental cache switch",
    },
    {
      flags: "--output-root <dir>",
      description: "Output root directory for generated files",
    },
    {
      flags: "--terminal <id>",
      description: "Terminal ID to generate (repeatable for multiple terminals)",
      required: true,
    },
    {
      flags: "--client-mode <mode>",
      description: "API client import mode: global (default) | local | package",
      defaultValue: "global",
    },
    {
      flags: "--client-path <path>",
      description: "Relative path to local client file (for --client-mode=local)",
    },
    {
      flags: "--client-package <name>",
      description: "Package name for custom client (for --client-mode=package)",
    },
    {
      flags: "--client-import-name <name>",
      description: "Custom import name (default: getApiClient)",
    },
  ];

  return {
    name: "codegen:run",
    summary: "Run code generation with specified terminals",
    description:
      "Orchestrates multi-terminal code generation from an OpenAPI specification.",
    options,
    examples: [
      "aptx-ft codegen run -i openapi.json --terminal axios-ts --output-root ./src",
      "aptx-ft codegen run -i openapi.json --terminal axios-ts --terminal react-query",
      "aptx-ft codegen run -i https://api.example.com/openapi.json --terminal functions",
    ],
    handler: async (ctx: PluginContext, args: Record<string, unknown>) => {
      const runOptions: CodegenRunOptions = {
        input: args.input as string | undefined,
        dryRun: args.dryRun as boolean,
        profile: args.profile as boolean,
        reportJson: args.reportJson as string | undefined,
        concurrencyOverride: args.concurrency as "auto" | number | undefined,
        cacheOverride:
          typeof args.cache === "string"
            ? parseBooleanLike(args.cache as string)
            : undefined,
        outputRoot: args.outputRoot as string | undefined,
        terminals: Array.isArray(args.terminal)
          ? (args.terminal as string[])
          : args.terminal
            ? [args.terminal as string]
            : [],
        clientMode: args.clientMode as "global" | "local" | "package" | undefined,
        clientPath: args.clientPath as string | undefined,
        clientPackage: args.clientPackage as string | undefined,
        clientImportName: args.clientImportName as string | undefined,
      };

      await runCodegen(runOptions);
    },
  };
}
