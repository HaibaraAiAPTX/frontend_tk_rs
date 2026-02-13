import path from "path";
import type { PluginContext, CommandHandler } from "@aptx/frontend-tk-core";

/**
 * Run model:enum-plan command
 * Exports enum enrichment plan JSON from model IR
 */
export const runModelEnumPlan: CommandHandler = async (
  ctx: PluginContext,
  args: Record<string, unknown>,
): Promise<void> => {
  const output = args.output as string;
  if (!output) {
    throw new Error("`--output` is required for model:enum-plan");
  }

  const outputAbsolutePath = path.resolve(process.cwd(), output);

  ctx.log(`Generating enum plan to: ${outputAbsolutePath}`);

  ctx.binding.runCli({
    input: args.input as string,
    command: "model:enum-plan",
    options: ["--output", outputAbsolutePath],
  });

  ctx.log("Enum plan generated successfully");
};

/**
 * Command descriptor for model:enum-plan
 */
export const enumPlanCommand = {
  name: "model:enum-plan",
  summary: "Export enum enrichment plan JSON from model IR",
  description: "Generates an enum enrichment plan JSON file that can be edited and applied via model:enum-apply.",
  options: [
    {
      flags: "-o, --output <file>",
      description: "Output JSON file path for the enum plan",
      required: true,
    },
  ],
  examples: [
    "aptx-ft model enum-plan -o enum-plan.json",
    "aptx-ft model enum-plan -o ./plans/enums.json",
  ],
  handler: runModelEnumPlan,
};
