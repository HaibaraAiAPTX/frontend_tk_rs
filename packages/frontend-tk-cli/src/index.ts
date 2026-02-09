import path from "path";
import fs from "fs";
import { getHelpTree, runCli } from "@aptx/frontend-tk-binding";
import { getConfig } from "./config";
import { getInput } from "./command/gen/get-input";
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

type CodegenConfig = {
  outputRoot?: string;
  terminals?: TerminalConfig[];
  modelOutput?: string;
};

type AppConfig = {
  input?: string;
  plugin?: string[];
  codegen?: CodegenConfig;
  performance?: {
    concurrency?: "auto" | number;
    cache?: boolean;
    format?: "fast" | "safe" | "strict";
  };
};

type GlobalOptions = {
  configFile?: string;
  input?: string;
  plugins: string[];
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
  }) => void | Promise<void>;
};

type ScriptPluginRenderer = {
  id: string;
  render: (ctx: {
    input: string;
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

const SUPPORTED_TERMINALS = [
  { id: "axios-ts", status: "available", mode: "axios-ts" },
  { id: "axios-js", status: "available", mode: "axios-js" },
  { id: "uniapp", status: "available", mode: "uniapp" },
  { id: "functions", status: "planned", mode: "" },
  { id: "react-query", status: "planned", mode: "" },
  { id: "vue-query", status: "planned", mode: "" },
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

function parseGlobalOptions(rawArgs: string[]): {
  global: GlobalOptions;
  commandTokens: string[];
} {
  const global: GlobalOptions = { plugins: [] };
  const commandTokens: string[] = [];
  for (let index = 0; index < rawArgs.length; index++) {
    const token = rawArgs[index];
    if (token === "-c" || token === "--config") {
      const value = rawArgs[index + 1];
      if (value && !value.startsWith("-")) {
        global.configFile = value;
        index += 1;
      }
      continue;
    }

    if (token === "-i" || token === "--input") {
      const value = rawArgs[index + 1];
      if (value && !value.startsWith("-")) {
        global.input = value;
        index += 1;
      }
      continue;
    }

    if (token === "-p" || token === "--plugin") {
      while (rawArgs[index + 1] && !rawArgs[index + 1].startsWith("-")) {
        global.plugins.push(rawArgs[index + 1]);
        index += 1;
      }
      continue;
    }

    commandTokens.push(token);
  }

  return {
    global,
    commandTokens,
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

function mapTerminalToLegacyMode(terminalId: string): string | undefined {
  const entry = SUPPORTED_TERMINALS.find((item) => item.id === terminalId);
  return entry && entry.mode ? entry.mode : undefined;
}

function printRootHelp() {
  console.log("aptx-ft");
  console.log("New CLI command set:");
  console.log("  codegen run");
  console.log("  codegen list-terminals");
  console.log("  doctor");
  console.log("  plugin list");
  console.log("Use `aptx-ft <command> --help` for details.");
}

function printCodegenRunHelp() {
  console.log("aptx-ft codegen run");
  console.log("Run code generation based on config `codegen` section.");
  console.log("Global options:");
  console.log("  -c, --config <file>   Config file path");
  console.log("  -i, --input <path>    Override input OpenAPI path/url");
  console.log("  -p, --plugin <paths>  Extra plugin dll paths");
}

function printCodegenListTerminalsHelp() {
  console.log("aptx-ft codegen list-terminals");
  console.log("List terminal support status.");
}

function printDoctorHelp() {
  console.log("aptx-ft doctor");
  console.log("Check runtime, binding and command registry status.");
}

function printPluginListHelp() {
  console.log("aptx-ft plugin list");
  console.log("List loaded command providers (built-in + plugins).");
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
  const loaded = await getConfig(global.configFile);
  const config = (loaded.config || {}) as AppConfig;
  const plugins = [...new Set([...(global.plugins || []), ...(config.plugin || [])])].map((item) =>
    ensureAbsolutePath(item),
  );
  const pluginGroups = splitPluginPaths(plugins);
  const scriptPlugins = await loadScriptPlugins(pluginGroups.scriptPlugins);
  const input = global.input || config.input;
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

async function runCodegen(global: GlobalOptions): Promise<void> {
  const { config, nativePlugins, scriptPlugins, input } = await loadMergedConfig(global);
  if (!input) {
    throw new Error("`input` is required. Use -i or set config.input.");
  }

  const codegen = config.codegen;
  if (!codegen) {
    throw new Error("`codegen` config is required for `codegen run`.");
  }

  const outputRoot = ensureAbsolutePath(codegen.outputRoot || "./generated");
  const terminals = codegen.terminals || [];
  if (!terminals.length) {
    throw new Error("`codegen.terminals` must contain at least one terminal.");
  }

  const resolvedInput = await getInput(input);
  const legacyArgs: string[] = [];
  const scriptTerminalTasks: Array<{ terminal: TerminalConfig; terminalId: string }> = [];
  for (const terminal of terminals) {
    const terminalId = resolveTerminalId(terminal);
    const legacyMode = mapTerminalToLegacyMode(terminalId);
    if (!legacyMode) {
      scriptTerminalTasks.push({ terminal, terminalId });
      continue;
    }
    legacyArgs.push("--service-mode", legacyMode);
    legacyArgs.push("--service-output", resolveTerminalOutput(terminal, outputRoot, terminalId));
  }

  if (legacyArgs.length) {
    const modelOutput = ensureAbsolutePath(
      codegen.modelOutput || path.join(outputRoot, "models"),
    );
    legacyArgs.push("--model-output", modelOutput);

    runCli({
      input: resolvedInput,
      command: "gen",
      options: legacyArgs,
      plugin: nativePlugins,
    });
  }

  if (scriptTerminalTasks.length) {
    for (const task of scriptTerminalTasks) {
      const scriptRenderer = findScriptRenderer(scriptPlugins, task.terminalId);
      if (!scriptRenderer) {
        throw new Error(
          `Terminal \`${task.terminalId}\` is not supported by built-in generator and no script renderer registered.`,
        );
      }
      const output = resolveTerminalOutput(task.terminal, outputRoot, task.terminalId);
      const writeFile = (filePath: string, content: string) => {
        const absolute = ensureAbsolutePath(path.join(output, filePath));
        fs.mkdirSync(path.dirname(absolute), { recursive: true });
        fs.writeFileSync(absolute, content);
      };
      const writeFiles = (files: Array<{ path: string; content: string }>) => {
        files.forEach((file) => writeFile(file.path, file.content));
      };

      try {
        await scriptRenderer.renderer.render({
          input: resolvedInput,
          terminal: task.terminal,
          outputRoot: output,
          config,
          writeFile,
          writeFiles,
        });
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        throw new Error(
          `script renderer failed: ${scriptRenderer.plugin.descriptor.pluginName}/${task.terminalId}: ${message}`,
        );
      }
    }
  }
}

async function listTerminals(): Promise<void> {
  console.log("Terminals:");
  SUPPORTED_TERMINALS.forEach((item) => {
    const mode = item.mode ? `legacy-mode=${item.mode}` : "legacy-mode=n/a";
    console.log(`  ${item.id} [${item.status}] ${mode}`);
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

  const scriptCommand = findScriptCommand(scriptPlugins, command);
  if (scriptCommand) {
    try {
      await scriptCommand.command.run({ args, input, config });
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

function printLegacyCommandMigration() {
  errorLog("`gen` command is deprecated in the new CLI.");
  console.log("Use `aptx-ft codegen run` instead.");
}

async function handleHelp(global: GlobalOptions, commandTokens: string[]): Promise<void> {
  const noCommand = commandTokens.length === 0;
  const [cmd1, cmd2] = commandTokens;

  if (noCommand) {
    printRootHelp();
    return;
  }

  if (cmd1 === "codegen" && cmd2 === "run") {
    printCodegenRunHelp();
    return;
  }
  if (cmd1 === "codegen" && cmd2 === "list-terminals") {
    printCodegenListTerminalsHelp();
    return;
  }
  if (cmd1 === "doctor") {
    printDoctorHelp();
    return;
  }
  if (cmd1 === "plugin" && cmd2 === "list") {
    printPluginListHelp();
    return;
  }
  if (cmd1 === "gen") {
    printLegacyCommandMigration();
    return;
  }

  const { nativePlugins, scriptPlugins } = await loadMergedConfig(global);
  const commands = [
    ...getHelpTreeSafe(nativePlugins),
    ...collectScriptCommandHelp(scriptPlugins),
  ];
  const found = commands.find(
    (item) => item.name === cmd1 || (item.aliases || []).includes(cmd1),
  );
  if (!found) {
    errorLog(`command not found: ${cmd1}`);
    printPluginCommands(commands);
    return;
  }
  printCommandDetail(found);
}

async function main() {
  const rawArgs = process.argv.slice(2);
  const { global, commandTokens } = parseGlobalOptions(rawArgs);
  const helpRequested = commandTokens.includes("--help") || commandTokens.includes("-h");

  if (helpRequested) {
    const commandOnly = commandTokens.filter((v) => v !== "--help" && v !== "-h");
    await handleHelp(global, commandOnly);
    return;
  }

  const [cmd1, cmd2, ...rest] = commandTokens;
  if (!cmd1) {
    printRootHelp();
    return;
  }

  if (cmd1 === "codegen" && cmd2 === "run") {
    await runCodegen(global);
    return;
  }
  if (cmd1 === "codegen" && cmd2 === "list-terminals") {
    await listTerminals();
    return;
  }
  if (cmd1 === "doctor") {
    await runDoctor(global);
    return;
  }
  if (cmd1 === "plugin" && cmd2 === "list") {
    await listPlugins(global);
    return;
  }
  if (cmd1 === "gen") {
    printLegacyCommandMigration();
    process.exitCode = 1;
    return;
  }

  await runNativeCommand(cmd1, [cmd2, ...rest].filter(Boolean), global);
}

main().catch((error: unknown) => {
  const message = error instanceof Error ? error.message : String(error);
  errorLog(message);
  process.exitCode = 1;
});
