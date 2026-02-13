import { PluginContext, CommandHandler } from "@aptx/frontend-tk-core";
import { runCli } from "@aptx/frontend-tk-binding";
import { ensureAbsolutePath } from "./utils";

export interface ModelGenOptions {
  output: string;
  style?: "declaration" | "module";
  name?: string[];
  input?: string;
}

/**
 * model:gen command handler - Generate TypeScript model files from OpenAPI schemas
 */
export const modelGenHandler: CommandHandler = async (
  ctx: PluginContext,
  args: Record<string, unknown>,
) => {
  const options = args as unknown as ModelGenOptions;

  if (!options.input) {
    throw new Error("`input` is required. Use -i or set config.input.");
  }

  if (!options.output) {
    throw new Error("`--output` is required.");
  }

  const cliOptions = [
    "--output",
    ensureAbsolutePath(options.output),
    "--style",
    options.style || "declaration",
  ];

  if (options.name && options.name.length > 0) {
    for (const name of options.name) {
      cliOptions.push("--name", name);
    }
  }

  runCli({
    input: options.input,
    command: "model:gen",
    options: cliOptions,
    plugin: [],
  });
};

/**
 * Command descriptor for model:gen
 */
export const modelGenCommand = {
  name: "model:gen",
  summary: "Generate TypeScript model files from OpenAPI schemas",
  description: "Generates TypeScript model declaration or module files from OpenAPI schema definitions.",
  options: [
    {
      flags: "-o, --output <dir>",
      description: "Output directory for generated model files",
      required: true,
    },
    {
      flags: "--style <declaration|module>",
      description: "Model output style (default: declaration)",
      defaultValue: "declaration",
    },
    {
      flags: "--name <schema>",
      description: "Generate specific schema names only; repeatable",
    },
  ],
  examples: [
    "aptx-ft model gen -o ./src/models",
    "aptx-ft model gen -o ./src/models --style module",
    "aptx-ft model gen -o ./src/models --name User --name Product",
  ],
  handler: modelGenHandler,
};
