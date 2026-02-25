import type {
  Plugin,
  PluginDescriptor,
  CommandDescriptor,
  CommandHandler,
  OptionDescriptor,
} from "@aptx/frontend-tk-core";
import { runInputDownload } from "./download";

/**
 * Plugin descriptor
 */
const descriptor: PluginDescriptor = {
  name: "@aptx/frontend-tk-plugin-input",
  version: "0.1.0",
  namespaceDescription: "Input handling (download)",
};

/**
 * Command options for input:download
 */
const inputDownloadOptions: OptionDescriptor[] = [
  {
    flags: "--url <url>",
    description: "OpenAPI JSON URL to download",
    required: true,
  },
  {
    flags: "--output <file>",
    description: "Output JSON file path",
    required: true,
  },
];

/**
 * Command handler for input:download
 */
const inputDownloadHandler: CommandHandler = async (
  _ctx,
  args,
): Promise<void> => {
  const url = args.url as string;
  const output = args.output as string;

  if (!url) {
    throw new Error("--url option is required");
  }
  if (!output) {
    throw new Error("--output option is required");
  }

  const result = await runInputDownload({ url, output });
  console.log(JSON.stringify(result, null, 2));
};

/**
 * Command descriptor for input:download
 */
const inputDownloadCommand: CommandDescriptor = {
  name: "input:download",
  summary: "Download OpenAPI JSON from URL to local file",
  description: "Downloads an OpenAPI JSON specification from a URL and saves it to a local file.",
  requiresOpenApi: false,
  options: inputDownloadOptions,
  examples: [
    "aptx-ft input download --url https://api.example.com/openapi.json --output ./swagger.json",
  ],
  handler: inputDownloadHandler,
};

/**
 * Plugin exports
 */
const plugin: Plugin = {
  descriptor,
  commands: [inputDownloadCommand],
};

export default plugin;
