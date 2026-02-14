import path from "path";
import type { PluginContext, CommandHandler } from "@aptx/frontend-tk-core";

/**
 * Run material:enum-plan command
 * Exports enum enrichment plan JSON from model IR
 */
export const runModelEnumPlan: CommandHandler = async (
  ctx: PluginContext,
  args: Record<string, unknown>,
): Promise<void> => {
  const output = args.output as string;
  if (!output) {
    throw new Error("`--output` is required for material:enum-plan");
  }

  const outputAbsolutePath = path.resolve(process.cwd(), output);
  const modelOutput = args.modelOutput as string | undefined;
  const modelOutputAbsolutePath = modelOutput
    ? path.resolve(process.cwd(), modelOutput)
    : undefined;

  ctx.log(`Generating enum plan to: ${outputAbsolutePath}`);

  const options = ["--output", outputAbsolutePath];
  if (modelOutputAbsolutePath) {
    options.push("--model-output", modelOutputAbsolutePath);
  }

  ctx.binding.runCli({
    input: args.input as string,
    command: "model:enum-plan",
    options,
  });

  ctx.log("Enum plan generated successfully");
};

/**
 * Command descriptor for material:enum-plan
 */
export const enumPlanCommand = {
  name: "material:enum-plan",
  summary: "Export enum enrichment plan JSON from model IR",
  description: "Generates an enum enrichment plan JSON file that can be edited and applied via material:enum-apply.",
  options: [
    {
      flags: "-o, --output <file>",
      description: "Output JSON file path for the enum plan",
      required: true,
    },
    {
      flags: "--model-output <dir>",
      description: "Existing generated model directory used to reuse translated enum names",
      required: false,
    },
  ],
  examples: [
    "aptx-ft material enum-plan -o enum-plan.json",
    "aptx-ft material enum-plan -o ./plans/enums.json",
    "aptx-ft material enum-plan -o ./tmp/enum-plan.json --model-output ./src/models",
  ],
  handler: runModelEnumPlan,
};
