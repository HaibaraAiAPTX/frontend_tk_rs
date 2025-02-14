import { Command } from "commander";
import { version } from '../package.json'

const program = new Command();

program
  .name('aptx-ft')
  .description("aptx-ft 是一个使用 rust 编写的根据 swagger 生成前端需要的模型、请求服务及部分界面的工具库，可根据项目需要编写 rust 插件进行自定义")
  .version(version);

program
  .command('gen')
  .description("gen 命令用于生成请求服务及模型")
  .requiredOption('-i, --input <string>', 'open_api 文件路径或 url')
  .option('-p, --plugin <string>', '自定义 rust 插件的位置')
  .option('--mo, --model-output [string...]', '模型输出目录')
  .option('--so, --service-output [string...]', '服务输入目录')
  .option('--sm, --service-mode [string...]', '生成服务的模式')
  .action(options => {
    import('./command/gen').then(v => {
      v.gen(options)
    })
  });

program.parse();