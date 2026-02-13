import path from "path";
import fs from "fs";
import os from "os";
import crypto from "crypto";
import { Command, InvalidArgumentError, Option } from "commander";
import { getHelpTree, runCli } from "@aptx/frontend-tk-binding";
import { getInput } from "./command/input/get-input";
import { ensureAbsolutePath, errorLog } from "./utils";

type HelpOptionDescriptor = {
  long: string;
  short?: string;
  valueName?: string;
  value_name?: string;
  required: boolean;
  multiple: boolean;
  defaultValue?: string;
  default_value?: string;
  description: string;
};

type HelpCommandDescriptor = {
  name: string;
  summary: string;
  description?: string;
  aliases: string[];
  options: HelpOptionDescriptor[];
  examples: string[];
  pluginName?: string;
  plugin_name?: string;
  pluginVersion?: string;
  plugin_version?: string;
};

type TerminalConfig = string | { id: string; output?: string };

type ClientImportConfig = {
  mode: "global" | "local" | "package";
  clientPath?: string;
  clientPackage?: string;
  importName?: string;
};

type CodegenConfig = {
  outputRoot?: string;
  terminals?: TerminalConfig[];
  clientImport?: ClientImportConfig;
};

type AppConfig = {
  input?: string;
  plugin?: string[];
  codegen?: CodegenConfig;
  scriptPluginPolicy?: {
    timeoutMs?: number;
    maxWriteFiles?: number;
    maxWriteBytes?: number;
    maxHeapMb?: number;
  };
  performance?: {
    concurrency?: "auto" | number;
    cache?: boolean;
  };
};

type ScriptPluginPolicy = {
  timeoutMs: number;
  maxWriteFiles: number;
  maxWriteBytes: number;
  maxHeapMb: number;
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
};

type ModelGenRunOptions = {
  output: string;
  style: "declaration" | "module";
  names: string[];
};

type ModelIrRunOptions = {
  output: string;
};

type InputDownloadRunOptions = {
  url: string;
  output: string;
};

type ModelEnumPlanRunOptions = {
  output: string;
};

type ModelEnumApplyRunOptions = {
  output: string;
  patch: string;
  style: "declaration" | "module";
  conflictPolicy: "openapi-first" | "patch-first" | "provider-first";
  names: string[];
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

type GlobalOptions = {
  input?: string;
  plugins: string[];
};

type GlobalCliOptions = {
  input?: string;
  plugin?: string[];
};

type ScriptPluginDescriptor = {
  apiVersion: string;
  pluginName: string;
  pluginVersion: string;
};

type ScriptPluginCommand = {
  name: string;
  summary?: string;
  description?: string;
  options?: HelpOptionDescriptor[];
  examples?: string[];
  run: (ctx: {
    args: string[];
    input?: string;
    config: AppConfig;
    getIrSnapshot: () => Promise<GeneratorInputIR>;
  }) => void | Promise<void>;
};

type ScriptPluginRenderer = {
  id: string;
  render: (ctx: {
    input: string;
    ir: GeneratorInputIR;
    terminal: TerminalConfig;
    outputRoot: string;
    config: AppConfig;
    writeFile: (filePath: string, content: string) => void;
    writeFiles: (files: Array<{ path: string; content: string }>) => void;
  }) => void | Promise<void>;
};

type ScriptPluginModule = ScriptPluginDescriptor & {
  commands?: ScriptPluginCommand[];
  renderers?: ScriptPluginRenderer[];
};

type LoadedScriptPlugin = {
  descriptor: ScriptPluginDescriptor;
  commands: ScriptPluginCommand[];
  renderers: ScriptPluginRenderer[];
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

const SUPPORTED_TERMINALS = [
  { id: "axios-ts", status: "available" },
  { id: "axios-js", status: "available" },
  { id: "uniapp", status: "available" },
  { id: "functions", status: "available" },
  { id: "react-query", status: "available" },
  { id: "vue-query", status: "available" },
];

function isScriptPluginPath(pluginPath: string): boolean {
  const ext = path.extname(pluginPath).toLowerCase();
  return ext === ".js" || ext === ".cjs" || ext === ".mjs";
}

function splitPluginPaths(pluginPaths: string[]): {
  nativePlugins: string[];
  scriptPlugins: string[];
} {
  const nativePlugins: string[] = [];
  const scriptPlugins: string[] = [];
  for (const pluginPath of pluginPaths) {
    const absolute = ensureAbsolutePath(pluginPath);
    if (isScriptPluginPath(absolute)) {
      scriptPlugins.push(absolute);
    } else {
      nativePlugins.push(absolute);
    }
  }
  return { nativePlugins, scriptPlugins };
}

function validateScriptPluginModule(
  module: Partial<ScriptPluginModule>,
  pluginPath: string,
): ScriptPluginModule {
  if (module.apiVersion !== "1") {
    throw new Error(`script plugin apiVersion mismatch: ${pluginPath}`);
  }
  if (!module.pluginName || !module.pluginVersion) {
    throw new Error(`script plugin missing pluginName/pluginVersion: ${pluginPath}`);
  }
  return {
    apiVersion: module.apiVersion,
    pluginName: module.pluginName,
    pluginVersion: module.pluginVersion,
    commands: module.commands || [],
    renderers: module.renderers || [],
  };
}

async function loadScriptPlugins(pluginPaths: string[]): Promise<LoadedScriptPlugin[]> {
  const loaded: LoadedScriptPlugin[] = [];
  for (const pluginPath of pluginPaths) {
    try {
      const rawModule = require(pluginPath);
      const moduleValue = (rawModule.default || rawModule) as Partial<ScriptPluginModule>;
      const module = validateScriptPluginModule(moduleValue, pluginPath);
      loaded.push({
        descriptor: {
          apiVersion: module.apiVersion,
          pluginName: module.pluginName,
          pluginVersion: module.pluginVersion,
        },
        commands: module.commands || [],
        renderers: module.renderers || [],
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      throw new Error(`failed to load script plugin ${pluginPath}: ${message}`);
    }
  }
  return loaded;
}

function parseGlobalOptionsWithCommander(rawArgs: string[]): {
  global: GlobalOptions;
  commandTokens: string[];
} {
  const parser = new Command()
    .exitOverride()
    .allowUnknownOption(true)
    .option("-i, --input <path>", "Override input OpenAPI path/url")
    .addOption(new Option("-p, --plugin <paths...>", "Extra plugin dll paths"));
  const parsed = parser.parseOptions(rawArgs);
  const options = parser.opts<GlobalCliOptions>();
  return {
    global: {
      input: options.input,
      plugins: options.plugin || [],
    },
    commandTokens: [...parsed.operands, ...parsed.unknown],
  };
}

function parseConcurrencyOption(value: string): "auto" | number {
  if (value === "auto") {
    return "auto";
  }
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    throw new InvalidArgumentError("`--concurrency` expects `auto` or a positive integer.");
  }
  return parsed;
}

function parseCacheOption(value: string): boolean {
  const parsed = parseBooleanLike(value);
  if (parsed === undefined) {
    throw new InvalidArgumentError("`--cache` expects true/false.");
  }
  return parsed;
}

function parseModelStyle(value: string): "declaration" | "module" {
  if (value === "declaration" || value === "module") {
    return value;
  }
  throw new InvalidArgumentError("`--style` expects declaration|module.");
}

function getGlobalOptionsFromCli(
  command: Command,
): GlobalOptions {
  const options = command.optsWithGlobals<GlobalCliOptions>();
  return {
    input: options.input,
    plugins: options.plugin || [],
  };
}

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

function printPluginCommands(commands: HelpCommandDescriptor[]) {
  if (!commands.length) {
    console.log("No plugin commands found.");
    return;
  }
  console.log("Commands:");
  commands.forEach((command) => {
    const from = command.pluginName || command.plugin_name || "unknown";
    const summary = command.summary ? ` - ${command.summary}` : "";
    console.log(`  ${command.name}${summary} [${from}]`);
  });
}

function printCommandDetail(command: HelpCommandDescriptor) {
  console.log(`${command.name}${command.summary ? ` - ${command.summary}` : ""}`);
  if (command.description) {
    console.log(command.description);
  }
  const from = command.pluginName || command.plugin_name;
  const version = command.pluginVersion || command.plugin_version;
  if (from || version) {
    console.log(`Source: ${from || "unknown"}${version ? `@${version}` : ""}`);
  }
  if (command.options?.length) {
    console.log("Options:");
    command.options.forEach((opt) => {
      const short = opt.short ? `-${opt.short}, ` : "";
      const valueName = opt.valueName || opt.value_name;
      const valuePart = valueName ? ` <${valueName}>` : "";
      console.log(`  ${short}--${opt.long}${valuePart}`);
      if (opt.description) {
        console.log(`    ${opt.description}`);
      }
    });
  }
  if (command.examples?.length) {
    console.log("Examples:");
    command.examples.forEach((example) => console.log(`  ${example}`));
  }
}

function mapScriptCommandToHelp(
  plugin: LoadedScriptPlugin,
  command: ScriptPluginCommand,
): HelpCommandDescriptor {
  return {
    name: command.name,
    summary: command.summary || "",
    description: command.description,
    aliases: [],
    options: command.options || [],
    examples: command.examples || [],
    plugin_name: plugin.descriptor.pluginName,
    plugin_version: plugin.descriptor.pluginVersion,
  };
}

function collectScriptCommandHelp(scriptPlugins: LoadedScriptPlugin[]): HelpCommandDescriptor[] {
  const result: HelpCommandDescriptor[] = [];
  scriptPlugins.forEach((plugin) => {
    plugin.commands.forEach((command) => {
      result.push(mapScriptCommandToHelp(plugin, command));
    });
  });
  return result;
}

function findScriptCommand(
  scriptPlugins: LoadedScriptPlugin[],
  commandName: string,
): { plugin: LoadedScriptPlugin; command: ScriptPluginCommand } | undefined {
  for (const plugin of scriptPlugins) {
    for (const command of plugin.commands) {
      if (command.name === commandName) {
        return { plugin, command };
      }
    }
  }
  return undefined;
}

function findScriptRenderer(
  scriptPlugins: LoadedScriptPlugin[],
  rendererId: string,
): { plugin: LoadedScriptPlugin; renderer: ScriptPluginRenderer } | undefined {
  for (const plugin of scriptPlugins) {
    for (const renderer of plugin.renderers) {
      if (renderer.id === rendererId) {
        return { plugin, renderer };
      }
    }
  }
  return undefined;
}

async function loadMergedConfig(global: GlobalOptions): Promise<{
  config: AppConfig;
  plugins: string[];
  nativePlugins: string[];
  scriptPlugins: LoadedScriptPlugin[];
  input?: string;
}> {
  // Configuration is now assembled directly from CLI parameters
  const config: AppConfig = {
    input: global.input,
    plugin: global.plugins,
  };
  const plugins = [...new Set([...(global.plugins || [])])].map((item) =>
    ensureAbsolutePath(item),
  );
  const pluginGroups = splitPluginPaths(plugins);
  const scriptPlugins = await loadScriptPlugins(pluginGroups.scriptPlugins);
  const input = global.input;
  return {
    config,
    plugins,
    nativePlugins: pluginGroups.nativePlugins,
    scriptPlugins,
    input,
  };
}

function getHelpTreeSafe(nativePlugins: string[]): HelpCommandDescriptor[] {
  if (typeof getHelpTree !== "function") {
    return [];
  }
  return getHelpTree({
    plugin: nativePlugins.length ? nativePlugins : undefined,
  }) as HelpCommandDescriptor[];
}

function buildIrSnapshotByNativeCommand(input: string, nativePlugins: string[]): GeneratorInputIR {
  const tempPath = ensureAbsolutePath(
    path.join(os.tmpdir(), `aptx-ir-${Date.now()}-${Math.random().toString(36).slice(2)}.json`),
  );
  try {
    runCli({
      input,
      command: "ir:snapshot",
      options: ["--output", tempPath],
      plugin: nativePlugins,
    });
    const text = fs.readFileSync(tempPath, "utf-8");
    return JSON.parse(text) as GeneratorInputIR;
  } finally {
    if (fs.existsSync(tempPath)) {
      fs.rmSync(tempPath, { force: true });
    }
  }
}

function buildIrSnapshot(input: string, nativePlugins: string[]): GeneratorInputIR {
  return buildIrSnapshotByNativeCommand(input, nativePlugins);
}

function runBuiltInTerminalCodegen(
  terminalId: string,
  input: string,
  output: string,
  nativePlugins: string[],
  clientImport?: ClientImportConfig,
): void {
  const options = ["--terminal", terminalId, "--output", output];

  // Add client import options if provided
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
    plugin: nativePlugins,
  });
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

function resolveScriptPluginPolicy(config: AppConfig): ScriptPluginPolicy {
  const raw = config.scriptPluginPolicy || {};
  return {
    timeoutMs: Math.max(1000, raw.timeoutMs ?? 30_000),
    maxWriteFiles: Math.max(1, raw.maxWriteFiles ?? 10_000),
    maxWriteBytes: Math.max(1024, raw.maxWriteBytes ?? 100 * 1024 * 1024),
    maxHeapMb: Math.max(64, raw.maxHeapMb ?? 1024),
  };
}

async function runWithTimeout<T>(
  task: Promise<T>,
  timeoutMs: number,
  timeoutMessage: string,
): Promise<T> {
  let timeoutHandle: NodeJS.Timeout | undefined;
  const timeoutPromise = new Promise<T>((_, reject) => {
    timeoutHandle = setTimeout(() => reject(new Error(timeoutMessage)), timeoutMs);
  });
  try {
    return await Promise.race([task, timeoutPromise]);
  } finally {
    if (timeoutHandle) {
      clearTimeout(timeoutHandle);
    }
  }
}

function assertWithinRoot(root: string, target: string): void {
  const normalizedRoot = path.resolve(root);
  const normalizedTarget = path.resolve(target);
  const relative = path.relative(normalizedRoot, normalizedTarget);
  if (relative.startsWith("..") || path.isAbsolute(relative)) {
    throw new Error(`write target escapes output root: ${normalizedTarget}`);
  }
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
  const workers = Array.from({ length: Math.min(limit, tasks.length) }, async () => {
    while (true) {
      const current = cursor;
      cursor += 1;
      if (current >= tasks.length) {
        break;
      }
      results[current] = await tasks[current]();
    }
  });
  await Promise.all(workers);
  return results;
}

async function runCodegen(global: GlobalOptions, runOptions: CodegenRunOptions): Promise<void> {
  const { config, nativePlugins, scriptPlugins, input } = await loadMergedConfig(global);
  const scriptPolicy = resolveScriptPluginPolicy(config);
  if (!input) {
    throw new Error("`input` is required. Use -i or set config.input.");
  }

  // Use CLI parameters if provided, otherwise fall back to config
  const codegen = config.codegen || {};

  // CLI parameters take priority over config file
  const outputRoot = ensureAbsolutePath(runOptions.outputRoot || codegen.outputRoot || "./generated");

  // Convert CLI terminal strings to TerminalConfig format
  const cliTerminals: TerminalConfig[] = (runOptions.terminals || []).map(id => ({ id }));
  const configTerminals = codegen.terminals || [];
  const terminals = cliTerminals.length > 0 ? cliTerminals : configTerminals;

  // Build client import config from CLI or config
  const clientImport: ClientImportConfig | undefined = (() => {
    // CLI parameters take priority
    if (runOptions.clientMode) {
      const result: ClientImportConfig = { mode: runOptions.clientMode };
      if (runOptions.clientMode === "local" && runOptions.clientPath) {
        result.clientPath = runOptions.clientPath;
      }
      if (runOptions.clientMode === "package" && runOptions.clientPackage) {
        result.clientPackage = runOptions.clientPackage;
      }
      if (runOptions.clientImportName) {
        result.importName = runOptions.clientImportName;
      }
      return result;
    }
    // Fall back to config
    return codegen.clientImport;
  })();

  if (!terminals.length) {
    throw new Error("`--terminal` parameter is required (e.g., --terminal axios-ts). Use --terminal multiple times for multiple terminals.");
  }

  const resolvedInput = await getInput(input);
  let irSnapshotCache: GeneratorInputIR | undefined;
  const getIrSnapshot = async () => {
    if (!irSnapshotCache) {
      irSnapshotCache = buildIrSnapshot(resolvedInput, nativePlugins);
    }
    return irSnapshotCache;
  };
  const terminalReports: TerminalRunReport[] = [];
  const builtInTerminalTasks: Array<{ terminal: TerminalConfig; terminalId: string; output: string }> = [];
  const scriptTerminalTasks: Array<{ terminal: TerminalConfig; terminalId: string; output: string }> = [];
  for (const terminal of terminals) {
    const terminalId = resolveTerminalId(terminal);
    const output = resolveTerminalOutput(terminal, outputRoot, terminalId);
    if (isBuiltInTerminal(terminalId)) {
      builtInTerminalTasks.push({ terminal, terminalId, output });
    } else {
      scriptTerminalTasks.push({ terminal, terminalId, output });
    }
  }
  const concurrency = resolveConcurrency(
    runOptions.concurrencyOverride ?? config.performance?.concurrency,
  );
  const cacheEnabled = runOptions.cacheOverride ?? config.performance?.cache ?? false;
  const cachePayload = {
    nativePlugins,
    scriptPlugins: scriptPlugins.map((item) => item.descriptor),
    codegen,
    runOptions: {
      dryRun: runOptions.dryRun,
      concurrency,
    },
  };
  const cacheFile = getCacheFilePath(outputRoot);
  const cacheState = readCacheState(cacheFile);
  const cacheKey = buildCodegenCacheKey(resolvedInput, cachePayload);
  const requiredOutputs = [
    ...builtInTerminalTasks.map((item) => item.output),
    ...scriptTerminalTasks.map((item) => item.output),
  ];
  const outputsReady = requiredOutputs.every((item) => fs.existsSync(item));
  const cacheHit = cacheEnabled && !runOptions.dryRun && outputsReady && cacheState.key === cacheKey;

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
    if (runOptions.profile) {
      console.log(`[profile] codegen cache hit, skipped generation, duration=${report.durationMs}ms`);
    }
    if (runOptions.reportJson) {
      const reportFile = ensureAbsolutePath(runOptions.reportJson);
      ensureDirectoryForFile(reportFile);
      fs.writeFileSync(reportFile, JSON.stringify(report, null, 2));
      console.log(`report written: ${reportFile}`);
    }
    return;
  }

  if (builtInTerminalTasks.length) {
    const snapshot = await getIrSnapshot();
    const builtInRunTasks = builtInTerminalTasks.map((task) => async () => {
      const outputTarget = runOptions.dryRun
        ? ensureAbsolutePath(path.join(os.tmpdir(), `aptx-ir-${task.terminalId}-${Date.now()}`))
        : task.output;
      const started = Date.now();
      runBuiltInTerminalCodegen(task.terminalId, resolvedInput, outputTarget, nativePlugins, clientImport);
      const outputInventory = buildFileInventoryFromDirectory(outputTarget);
      const reportItem = buildTerminalReport(
        {
          terminalId: task.terminalId,
          channel: "native",
          output: outputTarget,
          plannedFiles: outputInventory.length,
          writtenFiles: runOptions.dryRun ? 0 : outputInventory.length,
          skippedFiles: 0,
          durationMs: Date.now() - started,
          dryRun: runOptions.dryRun,
        },
        outputInventory,
        snapshot,
      );
      if (runOptions.dryRun && fs.existsSync(outputTarget)) {
        fs.rmSync(outputTarget, { recursive: true, force: true });
      }
      return reportItem;
    });
    const builtInReports = await runWithConcurrency(builtInRunTasks, concurrency);
    terminalReports.push(...builtInReports);
  }

  if (scriptTerminalTasks.length) {
    const snapshot = await getIrSnapshot();
    const scriptRunTasks = scriptTerminalTasks.map((task) => async () => {
      const scriptRenderer = findScriptRenderer(scriptPlugins, task.terminalId);
      if (!scriptRenderer) {
        throw new Error(
          `Terminal \`${task.terminalId}\` is not supported by built-in generator and no script renderer registered.`,
        );
      }
      const started = Date.now();
      let plannedFiles = 0;
      let writtenFiles = 0;
      let skippedFiles = 0;
      let plannedBytes = 0;
      const plannedFileMap = new Map<string, string>();
      const writeFile = (filePath: string, content: string) => {
        plannedFiles += 1;
        plannedBytes += Buffer.byteLength(content, "utf-8");
        if (plannedFiles > scriptPolicy.maxWriteFiles) {
          throw new Error(
            `write file limit exceeded: ${plannedFiles} > ${scriptPolicy.maxWriteFiles}`,
          );
        }
        if (plannedBytes > scriptPolicy.maxWriteBytes) {
          throw new Error(
            `write byte limit exceeded: ${plannedBytes} > ${scriptPolicy.maxWriteBytes}`,
          );
        }
        const heapUsedMb = Math.round(process.memoryUsage().heapUsed / 1024 / 1024);
        if (heapUsedMb > scriptPolicy.maxHeapMb) {
          throw new Error(
            `heap usage limit exceeded: ${heapUsedMb}MB > ${scriptPolicy.maxHeapMb}MB`,
          );
        }
        plannedFileMap.set(filePath.replace(/\\/g, "/"), content);
        const absolute = ensureAbsolutePath(path.join(task.output, filePath));
        assertWithinRoot(task.output, absolute);
        if (fs.existsSync(absolute)) {
          const existing = fs.readFileSync(absolute, "utf-8");
          if (existing === content) {
            skippedFiles += 1;
            return;
          }
        }
        if (runOptions.dryRun) {
          return;
        }
        fs.mkdirSync(path.dirname(absolute), { recursive: true });
        fs.writeFileSync(absolute, content);
        writtenFiles += 1;
      };
      const writeFiles = (files: Array<{ path: string; content: string }>) => {
        files.forEach((file) => writeFile(file.path, file.content));
      };

      try {
        await runWithTimeout(
          Promise.resolve(
            scriptRenderer.renderer.render({
              input: resolvedInput,
              ir: snapshot,
              terminal: task.terminal,
              outputRoot: task.output,
              config,
              writeFile,
              writeFiles,
            }),
          ),
          scriptPolicy.timeoutMs,
          `script renderer timeout after ${scriptPolicy.timeoutMs}ms`,
        );
        const fileInventory = runOptions.dryRun
          ? Array.from(plannedFileMap.entries()).map(([filePath, content]) => ({
              relativePath: filePath,
              sizeBytes: Buffer.byteLength(content, "utf-8"),
              content,
            }))
          : buildFileInventoryFromDirectory(task.output);
        return buildTerminalReport(
          {
            terminalId: task.terminalId,
            channel: "script",
            output: task.output,
            plannedFiles,
            writtenFiles,
            skippedFiles,
            durationMs: Date.now() - started,
            dryRun: runOptions.dryRun,
          },
          fileInventory,
          snapshot,
        );
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        throw new Error(
          `script renderer failed: ${scriptRenderer.plugin.descriptor.pluginName}/${task.terminalId}: ${message}`,
        );
      }
    });
    const scriptReports = await runWithConcurrency(scriptRunTasks, concurrency);
    terminalReports.push(...scriptReports);
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
    dryRun: runOptions.dryRun,
    cacheEnabled,
    cacheHit: false,
    concurrency,
    startedAt: startedAt.toISOString(),
    durationMs: Date.now() - runStartAt,
    terminalReports,
    totals,
  };

  if (runOptions.profile) {
    console.log(
      `[profile] codegen completed: duration=${report.durationMs}ms planned=${totals.plannedFiles} written=${totals.writtenFiles} skipped=${totals.skippedFiles}`,
    );
    terminalReports.forEach((item) => {
      console.log(
        `[profile] ${item.channel}:${item.terminalId} duration=${item.durationMs}ms planned=${item.plannedFiles} written=${item.writtenFiles} skipped=${item.skippedFiles}`,
      );
    });
  }

  if (runOptions.reportJson) {
    const reportFile = ensureAbsolutePath(runOptions.reportJson);
    ensureDirectoryForFile(reportFile);
    fs.writeFileSync(reportFile, JSON.stringify(report, null, 2));
    console.log(`report written: ${reportFile}`);
  }

  if (cacheEnabled && !runOptions.dryRun) {
    writeCacheState(cacheFile, cacheKey, report);
  }
}

async function runModelGen(global: GlobalOptions, runOptions: ModelGenRunOptions): Promise<void> {
  const { input, nativePlugins } = await loadMergedConfig(global);
  if (!input) {
    throw new Error("`input` is required. Use -i or set config.input.");
  }

  const options = ["--output", ensureAbsolutePath(runOptions.output), "--style", runOptions.style];
  runOptions.names.forEach((name) => {
    options.push("--name", name);
  });

  runCli({
    input: await getInput(input),
    command: "model:gen",
    options,
    plugin: nativePlugins,
  });
}

async function runModelIr(global: GlobalOptions, runOptions: ModelIrRunOptions): Promise<void> {
  const { input, nativePlugins } = await loadMergedConfig(global);
  if (!input) {
    throw new Error("`input` is required. Use -i or set config.input.");
  }

  runCli({
    input: await getInput(input),
    command: "model:ir",
    options: ["--output", ensureAbsolutePath(runOptions.output)],
    plugin: nativePlugins,
  });
}

async function runInputDownload(
  _global: GlobalOptions,
  runOptions: InputDownloadRunOptions,
): Promise<void> {
  const downloaded = await getInput(runOptions.url);
  const output = ensureAbsolutePath(runOptions.output);
  ensureDirectoryForFile(output);
  fs.copyFileSync(downloaded, output);
  const stat = fs.statSync(output);
  console.log(
    JSON.stringify(
      {
        success: true,
        filePath: output,
        size: stat.size,
        url: runOptions.url,
      },
      null,
      2,
    ),
  );
}

async function runModelEnumPlan(
  global: GlobalOptions,
  runOptions: ModelEnumPlanRunOptions,
): Promise<void> {
  const { input, nativePlugins } = await loadMergedConfig(global);
  if (!input) {
    throw new Error("`input` is required. Use -i or set config.input.");
  }

  runCli({
    input: await getInput(input),
    command: "model:enum-plan",
    options: ["--output", ensureAbsolutePath(runOptions.output)],
    plugin: nativePlugins,
  });
}

async function runModelEnumApply(
  global: GlobalOptions,
  runOptions: ModelEnumApplyRunOptions,
): Promise<void> {
  const { input, nativePlugins } = await loadMergedConfig(global);
  if (!input) {
    throw new Error("`input` is required. Use -i or set config.input.");
  }

  const options = [
    "--output",
    ensureAbsolutePath(runOptions.output),
    "--patch",
    ensureAbsolutePath(runOptions.patch),
    "--style",
    runOptions.style,
    "--conflict-policy",
    runOptions.conflictPolicy,
  ];
  runOptions.names.forEach((name) => {
    options.push("--name", name);
  });

  runCli({
    input: await getInput(input),
    command: "model:enum-apply",
    options,
    plugin: nativePlugins,
  });
}

async function listTerminals(): Promise<void> {
  console.log("Terminals:");
  SUPPORTED_TERMINALS.forEach((item) => {
    console.log(`  ${item.id} [${item.status}]`);
  });
}

async function runDoctor(global: GlobalOptions): Promise<void> {
  const { nativePlugins, scriptPlugins } = await loadMergedConfig(global);
  const hasHelpTree = typeof getHelpTree === "function";
  console.log(`Node: ${process.version}`);
  console.log(`Binding getHelpTree: ${hasHelpTree ? "ok" : "missing"}`);
  const nativeCommands = getHelpTreeSafe(nativePlugins);
  const scriptCommands = collectScriptCommandHelp(scriptPlugins);
  const commands = [...nativeCommands, ...scriptCommands];
  console.log(`Registered commands: ${commands.length}`);
  if (commands.length) {
    commands.forEach((item) => console.log(`  - ${item.name}`));
  }
  console.log(`Script plugins: ${scriptPlugins.length}`);
}

async function listPlugins(global: GlobalOptions): Promise<void> {
  const { nativePlugins, scriptPlugins } = await loadMergedConfig(global);
  const commands = [
    ...getHelpTreeSafe(nativePlugins),
    ...collectScriptCommandHelp(scriptPlugins),
  ];
  if (!commands.length) {
    console.log("No command descriptors available.");
    return;
  }
  const grouped = new Map<string, number>();
  commands.forEach((command) => {
    const key = command.pluginName || command.plugin_name || "unknown";
    grouped.set(key, (grouped.get(key) || 0) + 1);
  });
  console.log("Plugin providers:");
  Array.from(grouped.entries())
    .sort(([a], [b]) => a.localeCompare(b))
    .forEach(([name, count]) => {
      console.log(`  ${name}: ${count} command(s)`);
    });
}

async function runNativeCommand(
  command: string,
  args: string[],
  global: GlobalOptions,
): Promise<void> {
  const { config, nativePlugins, scriptPlugins, input } = await loadMergedConfig(global);
  const scriptPolicy = resolveScriptPluginPolicy(config);

  const scriptCommand = findScriptCommand(scriptPlugins, command);
  if (scriptCommand) {
    let snapshot: GeneratorInputIR | undefined;
    let resolvedInputPath: string | undefined;
    const getIrSnapshot = async () => {
      if (!input) {
        throw new Error("`input` is required to build IR snapshot.");
      }
      if (!resolvedInputPath) {
        resolvedInputPath = await getInput(input);
      }
      if (!snapshot) {
        snapshot = buildIrSnapshot(resolvedInputPath, nativePlugins);
      }
      return snapshot;
    };
    try {
      await runWithTimeout(
        Promise.resolve(scriptCommand.command.run({ args, input, config, getIrSnapshot })),
        scriptPolicy.timeoutMs,
        `script command timeout after ${scriptPolicy.timeoutMs}ms`,
      );
      return;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      throw new Error(
        `script command failed: ${scriptCommand.plugin.descriptor.pluginName}/${command}: ${message}`,
      );
    }
  }

  if (!input) {
    throw new Error("`input` is required. Use -i or set config.input.");
  }
  runCli({
    input: await getInput(input),
    command,
    options: args,
    plugin: nativePlugins,
  });
}

async function handlePluginHelp(global: GlobalOptions, commandTokens: string[]): Promise<void> {
  const [commandName] = commandTokens;
  if (!commandName) {
    return;
  }
  const { nativePlugins, scriptPlugins } = await loadMergedConfig(global);
  const commands = [
    ...getHelpTreeSafe(nativePlugins),
    ...collectScriptCommandHelp(scriptPlugins),
  ];
  const found = commands.find(
    (item) => item.name === commandName || (item.aliases || []).includes(commandName),
  );
  if (!found) {
    errorLog(`command not found: ${commandName}`);
    printPluginCommands(commands);
    return;
  }
  printCommandDetail(found);
}

function buildProgram(): Command {
  const program = new Command();
  program
    .name("aptx-ft")
    .description("Frontend toolkit CLI for OpenAPI code generation.")
    .showHelpAfterError()
    .addOption(new Option("-i, --input <path>", "Override input OpenAPI path/url"))
    .addOption(new Option("-p, --plugin <paths...>", "Extra plugin dll paths"));

  const codegen = program
    .command("codegen")
    .description("Code generation related commands (e.g., codegen:run).");

  const model = program
    .command("model")
    .description("Model generation related commands (e.g., model:gen, model:ir).");

  const input = program
    .command("input")
    .description("Input related commands (e.g., input:download).");

  codegen
    .command("run")
    .alias("codegen:run")
    .description("Run code generation based on config `codegen` section (alias: codegen:run).")
    .option("--dry-run", "Build plan without writing files", false)
    .option("--profile", "Print execution timing summary", false)
    .option("--report-json <file>", "Write execution report JSON")
    .addOption(
      new Option("--concurrency <value>", "Override concurrency, e.g. auto/4").argParser(
        parseConcurrencyOption,
      ),
    )
    .addOption(
      new Option("--cache <true|false>", "Override incremental cache switch").argParser(
        parseCacheOption,
      ),
    )
    .addOption(
      new Option("--output-root <dir>", "Output root directory for generated files"),
    )
    .addOption(
      new Option("--terminal <id>", "Terminal ID to generate (repeatable for multiple terminals)")
        .default([])
        .argParser((value: string, previous: string[]) => [...previous, value]),
    )
    .addOption(
      new Option("--client-mode <mode>", "API client import mode: global (default) | local | package")
        .choices(["global", "local", "package"])
        .default("global"),
    )
    .addOption(
      new Option("--client-path <path>", "Relative path to local client file (for --client-mode=local)"),
    )
    .addOption(
      new Option("--client-package <name>", "Package name for custom client (for --client-mode=package)"),
    )
    .addOption(
      new Option("--client-import-name <name>", "Custom import name (default: getApiClient)"),
    )
    .action(async (options: {
      dryRun: boolean;
      profile: boolean;
      reportJson?: string;
      concurrency?: "auto" | number;
      cache?: boolean;
      outputRoot?: string;
      terminal?: string[];
      clientMode?: "global" | "local" | "package";
      clientPath?: string;
      clientPackage?: string;
      clientImportName?: string;
    }, command: Command) => {
      const global = getGlobalOptionsFromCli(command);
      const runOptions: CodegenRunOptions = {
        dryRun: options.dryRun,
        profile: options.profile,
        reportJson: options.reportJson,
        concurrencyOverride: options.concurrency,
        cacheOverride: options.cache,
        outputRoot: options.outputRoot,
        terminals: options.terminal,
        clientMode: options.clientMode,
        clientPath: options.clientPath,
        clientPackage: options.clientPackage,
        clientImportName: options.clientImportName,
      };
      await runCodegen(global, runOptions);
    });

  codegen
    .command("list-terminals")
    .alias("codegen:list-terminals")
    .description("List terminal support status (alias: codegen:list-terminals).")
    .action(async () => {
      await listTerminals();
    });

  model
    .command("gen")
    .alias("model:gen")
    .description("Generate TypeScript model files from OpenAPI schemas (alias: model:gen).")
    .requiredOption("--output <dir>", "Output directory")
    .addOption(
      new Option("--style <declaration|module>", "Model output style")
        .default("declaration")
        .argParser(parseModelStyle),
    )
    .addOption(
      new Option("--name <schema>", "Generate specific schema names only; repeatable")
        .default([])
        .argParser((value: string, previous: string[]) => [...previous, value]),
    )
    .action(async (options: { output: string; style: "declaration" | "module"; name?: string[] }, command: Command) => {
      const global = getGlobalOptionsFromCli(command);
      await runModelGen(global, {
        output: options.output,
        style: options.style,
        names: options.name || [],
      });
    });

  model
    .command("ir")
    .alias("model:ir")
    .description("Export model intermediate representation JSON (alias: model:ir).")
    .requiredOption("--output <file>", "Output JSON file path")
    .action(async (options: { output: string }, command: Command) => {
      const global = getGlobalOptionsFromCli(command);
      await runModelIr(global, {
        output: options.output,
      });
    });

  input
    .command("download")
    .alias("input:download")
    .description("Download OpenAPI JSON from URL to local file (alias: input:download).")
    .requiredOption("--url <url>", "OpenAPI JSON URL")
    .requiredOption("--output <file>", "Output JSON file path")
    .action(async (options: { url: string; output: string }, command: Command) => {
      const global = getGlobalOptionsFromCli(command);
      await runInputDownload(global, {
        url: options.url,
        output: options.output,
      });
    });

  model
    .command("enum-plan")
    .alias("model:enum-plan")
    .description("Export enum enrichment plan JSON from model IR (alias: model:enum-plan).")
    .requiredOption("--output <file>", "Output JSON file path")
    .action(async (options: { output: string }, command: Command) => {
      const global = getGlobalOptionsFromCli(command);
      await runModelEnumPlan(global, {
        output: options.output,
      });
    });

  model
    .command("enum-apply")
    .alias("model:enum-apply")
    .description("Apply enum patch JSON and generate model files (alias: model:enum-apply).")
    .requiredOption("--output <dir>", "Output directory")
    .requiredOption("--patch <file>", "Enum patch JSON file path")
    .addOption(
      new Option("--style <declaration|module>", "Model output style")
        .default("declaration")
        .argParser(parseModelStyle),
    )
    .addOption(
      new Option(
        "--conflict-policy <openapi-first|patch-first|provider-first>",
        "Conflict policy for enum merge",
      ).default("patch-first"),
    )
    .addOption(
      new Option("--name <schema>", "Generate specific schema names only; repeatable")
        .default([])
        .argParser((value: string, previous: string[]) => [...previous, value]),
    )
    .action(async (
      options: {
        output: string;
        patch: string;
        style: "declaration" | "module";
        conflictPolicy?: "openapi-first" | "patch-first" | "provider-first";
        name?: string[];
      },
      command: Command,
    ) => {
      const global = getGlobalOptionsFromCli(command);
      await runModelEnumApply(global, {
        output: options.output,
        patch: options.patch,
        style: options.style,
        conflictPolicy: options.conflictPolicy || "patch-first",
        names: options.name || [],
      });
    });

  program
    .command("doctor")
    .description("Check runtime, binding and command registry status.")
    .action(async (_: unknown, command: Command) => {
      const global = getGlobalOptionsFromCli(command);
      await runDoctor(global);
    });

  program
    .command("plugin")
    .description("Plugin related commands.")
    .command("list")
    .description("List loaded command providers (built-in + plugins).")
    .action(async (_: unknown, command: Command) => {
      const global = getGlobalOptionsFromCli(command);
      await listPlugins(global);
    });

  program.addHelpText(
    "after",
    `
Plugin/native commands: \`aptx-ft <command> [args...]\`

Built-in commands:
  codegen:run          Run code generation based on config or CLI options
  codegen:list-terminals List terminal support status
  model:gen            Generate TypeScript model declarations from OpenAPI schemas
  model:ir             Export model IR snapshot JSON from OpenAPI schemas
  model:enum-plan      Export enum enrichment plan JSON from model IR
  model:enum-apply     Apply enum patch file and generate model files
  input:download        Download OpenAPI JSON from URL to local file
  terminal:codegen     Generate output for one built-in terminal from OpenAPI input
  ir:snapshot          Export IR snapshot JSON from OpenAPI input

Use \`aptx-ft <command> --help\` for more details on a specific command.
`,
  );
  return program;
}

function isBuiltInCommand(command?: string): boolean {
  const builtInCategories = ["codegen", "doctor", "plugin", "model", "input"];
  // Check both category commands (e.g., "model") and full colon commands (e.g., "model:gen")
  if (builtInCategories.includes(command || "")) {
    return true;
  }
  // Check for colon-format built-in commands
  if (command && command.includes(":")) {
    const [category] = command.split(":");
    return builtInCategories.includes(category);
  }
  return false;
}

async function main() {
  const rawArgs = process.argv.slice(2);
  const { global, commandTokens } = parseGlobalOptionsWithCommander(rawArgs);
  const helpRequested = commandTokens.includes("--help") || commandTokens.includes("-h");
  const commandOnlyTokens = commandTokens.filter(
    (token) => token !== "--help" && token !== "-h",
  );
  const [commandName, ...restTokens] = commandOnlyTokens;
  const program = buildProgram();

  if (!commandName) {
    program.outputHelp();
    return;
  }

  if (isBuiltInCommand(commandName)) {
    await program.parseAsync(process.argv);
    return;
  }

  if (helpRequested) {
    await handlePluginHelp(global, [commandName, ...restTokens]);
    return;
  }

  await runNativeCommand(commandName, restTokens, global);
}

main().catch((error: unknown) => {
  const message = error instanceof Error ? error.message : String(error);
  errorLog(message);
  process.exitCode = 1;
});
