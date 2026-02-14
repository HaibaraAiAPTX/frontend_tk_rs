import path from "path";
import type { PluginContext, CommandHandler } from "@aptx/frontend-tk-core";

type NamingStrategy = "auto" | "none";

/**
 * Run materal:enum-patch command
 * Fetch enum key/value pairs from Materal backend endpoints.
 */
export const runMateralEnumPatch: CommandHandler = async (
  ctx: PluginContext,
  args: Record<string, unknown>,
): Promise<void> => {
  const baseUrl = args.baseUrl as string;
  const output = args.output as string;
  const maxRetries = args.maxRetries as number | undefined;
  const timeoutMs = args.timeoutMs as number | undefined;
  const namingStrategy = (args.namingStrategy as NamingStrategy | undefined) ?? "auto";

  if (!baseUrl) {
    throw new Error("`--base-url` is required for materal:enum-patch");
  }
  if (!output) {
    throw new Error("`--output` is required for materal:enum-patch");
  }

  const outputAbsolutePath = path.resolve(process.cwd(), output);
  ctx.log(`Fetching enum patch from Materal API: ${baseUrl}`);
  ctx.log(`Writing enum patch to: ${outputAbsolutePath}`);

  const options = [
    "--base-url",
    baseUrl,
    "--output",
    outputAbsolutePath,
    "--naming-strategy",
    namingStrategy,
  ];

  if (typeof maxRetries === "number") {
    options.push("--max-retries", String(maxRetries));
  }
  if (typeof timeoutMs === "number") {
    options.push("--timeout-ms", String(timeoutMs));
  }

  ctx.binding.runCli({
    input: args.input as string,
    command: "materal:enum-patch",
    options,
  });

  ctx.log("Materal enum patch generated successfully");
};

/**
 * Command descriptor for materal:enum-patch
 */
export const enumPatchCommand = {
  name: "materal:enum-patch",
  summary: "Fetch Materal enum values and output enum patch JSON",
  description: "Fetches enum key/value pairs from Materal backend endpoints and exports a standard enum patch JSON file.",
  options: [
    {
      flags: "-i, --input <path>",
      description: "OpenAPI spec file path or URL",
      required: false,
    },
    {
      flags: "--base-url <url>",
      description: "Materal backend API base URL",
      required: true,
    },
    {
      flags: "-o, --output <file>",
      description: "Output JSON file path for enum patch",
      required: true,
    },
    {
      flags: "--max-retries <n>",
      description: "HTTP retry count (default: 3)",
      defaultValue: "3",
    },
    {
      flags: "--timeout-ms <ms>",
      description: "HTTP timeout in milliseconds (default: 10000)",
      defaultValue: "10000",
    },
    {
      flags: "--naming-strategy <auto|none>",
      description: "Enum member naming strategy (default: auto)",
      defaultValue: "auto",
    },
  ],
  examples: [
    "aptx-ft -i ./openapi.json materal enum-patch --base-url http://localhost:5000 -o ./tmp/enum-patch.json",
    "aptx-ft -i ./openapi.json materal enum-patch --base-url http://localhost:5000 -o ./tmp/enum-patch.json --naming-strategy none",
  ],
  handler: runMateralEnumPatch,
};
