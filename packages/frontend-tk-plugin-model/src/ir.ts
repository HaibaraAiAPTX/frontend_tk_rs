import { PluginContext, CommandHandler } from "@aptx/frontend-tk-core";
import { runCli } from "@aptx/frontend-tk-binding";
import { ensureAbsolutePath } from "./utils";

export interface ModelIrOptions {
  output: string;
  input?: string;
}

/**
 * model:ir command handler - Export model intermediate representation JSON
 */
export const modelIrHandler: CommandHandler = async (
  ctx: PluginContext,
  args: Record<string, unknown>,
) => {
  const options = args as unknown as ModelIrOptions;

  if (!options.input) {
    throw new Error("`input` is required. Use -i or set config.input.");
  }

  if (!options.output) {
    throw new Error("`--output` is required.");
  }

  runCli({
    input: options.input,
    command: "model:ir",
    options: ["--output", ensureAbsolutePath(options.output)],
    plugin: [],
  });
};

/**
 * Command descriptor for model:ir
 */
export const modelIrCommand = {
  name: "model:ir",
  summary: "Export model intermediate representation JSON",
  description: "Exports the model intermediate representation (IR) as a JSON file for inspection or further processing.",
  options: [
    {
      flags: "-o, --output <file>",
      description: "Output JSON file path",
      required: true,
    },
  ],
  examples: [
    "aptx-ft model ir -o ./model-ir.json",
    "aptx-ft model ir -o ./snapshot/model-ir.json",
  ],
  handler: modelIrHandler,
};
