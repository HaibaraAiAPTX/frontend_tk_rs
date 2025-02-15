import { Command } from "commander";
import { getConfig } from "./config";
import { errorLog } from "./utils";
import { runCli } from '@aptx/frontend-tk-binding'
import { getInput } from "./command/gen/get-input";

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