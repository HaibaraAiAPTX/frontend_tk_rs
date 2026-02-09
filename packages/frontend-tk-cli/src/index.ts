import { Command } from "commander";
import { getConfig } from "./config";
import { errorLog } from "./utils";
import { getHelpTree, runCli } from '@aptx/frontend-tk-binding'
import { getInput } from "./command/gen/get-input";

type HelpOptionDescriptor = {
  long: string
  short?: string
  valueName?: string
  value_name?: string
  required: boolean
  multiple: boolean
  defaultValue?: string
  default_value?: string
  description: string
}

type HelpCommandDescriptor = {
  name: string
  summary: string
  description?: string
  aliases: string[]
  options: HelpOptionDescriptor[]
  examples: string[]
  pluginName?: string
  plugin_name?: string
  pluginVersion?: string
  plugin_version?: string
}

function parseMetaArgs(rawArgs: string[]) {
  let command: string | undefined;
  let config: string | undefined;
  const plugins: string[] = [];

  for (let i = 0; i < rawArgs.length; i++) {
    const token = rawArgs[i];
    if (token === "-c" || token === "--config") {
      const value = rawArgs[i + 1];
      if (value && !value.startsWith("-")) {
        config = value;
        i++;
      }
      continue;
    }
    if (token === "-p" || token === "--plugin") {
      while (rawArgs[i + 1] && !rawArgs[i + 1].startsWith("-")) {
        plugins.push(rawArgs[i + 1]);
        i++;
      }
      continue;
    }
    if (!token.startsWith("-") && !command) {
      command = token;
    }
  }

  return { command, config, plugins };
}

function printCommandList(commandList: HelpCommandDescriptor[]) {
  console.log("Commands:");
  commandList.forEach((command) => {
    const summary = command.summary || "";
    const from = command.pluginName || command.plugin_name || "unknown";
    console.log(`  ${command.name}${summary ? ` - ${summary}` : ""} [${from}]`);
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

  if (command.aliases?.length) {
    console.log(`Aliases: ${command.aliases.join(", ")}`);
  }

  if (command.options?.length) {
    console.log("Options:");
    command.options.forEach((opt) => {
      const short = opt.short ? `-${opt.short}, ` : "";
      const valueName = opt.valueName || opt.value_name;
      const defaultValue = opt.defaultValue || opt.default_value;
      const valuePart = valueName ? ` <${valueName}>` : "";
      const requiredTag = opt.required ? " (required)" : "";
      const defaultTag = defaultValue ? ` (default: ${defaultValue})` : "";
      console.log(`  ${short}--${opt.long}${valuePart}${requiredTag}${defaultTag}`);
      if (opt.description) {
        console.log(`    ${opt.description}`);
      }
    });
  }

  if (command.examples?.length) {
    console.log("Examples:");
    command.examples.forEach((example) => {
      console.log(`  ${example}`);
    });
  }
}

async function printDynamicHelp(rawArgs: string[]) {
  if (typeof getHelpTree !== "function") {
    console.log("Dynamic help is unavailable: native binding does not expose `getHelpTree` yet.");
    console.log("Please rebuild @aptx/frontend-tk-binding and retry.");
    console.log("Available built-in command:");
    console.log("  gen");
    return;
  }

  const { command, config, plugins: cliPlugins } = parseMetaArgs(rawArgs);
  const configData = await getConfig(config);
  const mergedPlugins = [...new Set([...(cliPlugins || []), ...(configData.config.plugin || [])])];
  const commandList = getHelpTree({
    plugin: mergedPlugins.length ? mergedPlugins : undefined
  }) as HelpCommandDescriptor[];

  if (command) {
    const found = commandList.find((item) => item.name === command || item.aliases?.includes(command));
    if (!found) {
      errorLog(`未找到命令: ${command}`);
      printCommandList(commandList);
      return;
    }
    printCommandDetail(found);
    return;
  }

  console.log("aptx-ft");
  console.log("动态命令帮助（内置 + 插件）");
  printCommandList(commandList);
  console.log("Use `aptx-ft <command> --help` for command details.");
}

const rawArgs = process.argv.slice(2);
if (rawArgs.includes("--help") || rawArgs.includes("-h")) {
  printDynamicHelp(rawArgs)
    .catch((e) => console.error(e))
    .finally(() => process.exit(0));
} else {
  const program = new Command();

  program
    .name('aptx-ft')
    .description("使用 rust 编写的根据 swagger 生成前端需要的模型、请求服务及部分界面的工具库，可根据项目需要编写 rust 插件进行自定义")
    .argument('<args...>', '要执行的命令')
    .allowUnknownOption(true)
    .option('-c, --config [string]', '配置文件路径')
    .option('-i, --input [string]', '入口 json 文件，可以为 url')
    .option('-p, --plugin [string...]', '要注册的 dll 文件路径')
    .action(async (args: string[]) => {
      try {
        const opts = program.opts<{ config?: string, input?: string, plugin?: string[] }>()

        const command = args.shift();
        if (!command) {
          errorLog('执行命令不能为空')
          return
        }

        const config = await getConfig(opts.config);

        if (!config.config.input && !opts.input) {
          errorLog('json 入口文件不能为空')
          return
        }

        let input = await getInput((opts.input || config.config.input)!)

        let plugins = [...new Set([...(opts.plugin || []), ...(config.config.plugin || [])])]

        runCli({
          input,
          command,
          options: args,
          plugin: plugins
        })
      } catch (e) {
        console.error(e);
      }
    })
    .parse()
}
