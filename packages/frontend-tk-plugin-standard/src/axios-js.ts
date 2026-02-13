import { PluginContext, CommandHandler } from "@aptx/frontend-tk-core";
import { runCli } from "@aptx/frontend-tk-binding";
import { ensureAbsolutePath } from "./utils";

export interface AxiosJsOptions {
  output: string;
  input?: string;
  clientMode?: "global" | "local" | "package";
  clientPath?: string;
  clientPackage?: string;
  clientImportName?: string;
}

/**
 * std:axios-js command handler - Generate Axios JavaScript function calls
 */
export const axiosJsHandler: CommandHandler = async (
  ctx: PluginContext,
  args: Record<string, unknown>,
) => {
  const options = args as unknown as AxiosJsOptions;

  if (!options.input) {
    throw new Error("`input` is required. Use -i or set config.input.");
  }

  if (!options.output) {
    throw new Error("`--output` is required.");
  }

  const cliOptions = [
    "--output",
    ensureAbsolutePath(options.output),
  ];

  // Add client import options if provided
  if (options.clientMode && options.clientMode !== "global") {
    cliOptions.push("--client-mode", options.clientMode);
  }
  if (options.clientMode === "local" && options.clientPath) {
    cliOptions.push("--client-path", ensureAbsolutePath(options.clientPath));
  }
  if (options.clientMode === "package" && options.clientPackage) {
    cliOptions.push("--client-package", options.clientPackage);
  }
  if (options.clientImportName) {
    cliOptions.push("--client-import-name", options.clientImportName);
  }

  runCli({
    input: options.input,
    command: "std:axios-js",
    options: cliOptions,
    plugin: [],
  });
};

/**
 * Command descriptor for std:axios-js
 */
export const axiosJsCommand = {
  name: "std:axios-js",
  summary: "Generate Axios JavaScript function calls",
  description: "Generates JavaScript function-style API calls using axios.",
  options: [
    {
      flags: "-o, --output <dir>",
      description: "Output directory for generated function files",
      required: true,
    },
    {
      flags: "--client-mode <global|local|package>",
      description: "API client import mode (default: global)",
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
  ],
  examples: [
    "aptx-ft std axios-js -o ./src/services",
    "aptx-ft std axios-js -o ./src/services --client-mode local --client-path ./src/api/client.js",
    "aptx-ft std axios-js -o ./src/services --client-mode package --client-package @myorg/api-client",
  ],
  handler: axiosJsHandler,
};
