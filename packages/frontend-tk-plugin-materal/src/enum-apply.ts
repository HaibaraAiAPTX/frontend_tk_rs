import path from "path";
import type { PluginContext, CommandHandler } from "@aptx/frontend-tk-core";

/**
 * Conflict policy options for enum merge
 */
type ConflictPolicy = "openapi-first" | "patch-first" | "provider-first";

/**
 * Model style options
 */
type ModelStyle = "declaration" | "module";

/**
 * Run materal:enum-apply command
 * Applies enum patch JSON and generates model files
 */
export const runModelEnumApply: CommandHandler = async (
  ctx: PluginContext,
  args: Record<string, unknown>,
): Promise<void> => {
  const output = args.output as string;
  const patch = args.patch as string;
  const style = (args.style as ModelStyle) ?? "declaration";
  const conflictPolicy = (args.conflictPolicy as ConflictPolicy) ?? "patch-first";
  const names = (args.names as string[]) ?? [];

  if (!output) {
    throw new Error("`--output` is required for materal:enum-apply");
  }
  if (!patch) {
    throw new Error("`--patch` is required for materal:enum-apply");
  }

  const outputAbsolutePath = path.resolve(process.cwd(), output);
  const patchAbsolutePath = path.resolve(process.cwd(), patch);

  ctx.log(`Applying enum patch from: ${patchAbsolutePath}`);
  ctx.log(`Generating models to: ${outputAbsolutePath}`);
  ctx.log(`Style: ${style}, Conflict policy: ${conflictPolicy}`);

  const options = [
    "--output",
    outputAbsolutePath,
    "--patch",
    patchAbsolutePath,
    "--style",
    style,
    "--conflict-policy",
    conflictPolicy,
  ];

  for (const name of names) {
    options.push("--name", name);
  }

  ctx.binding.runCli({
    input: args.input as string,
    command: "model:enum-apply",
    options,
  });

  ctx.log("Enum patch applied and models generated successfully");
};

/**
 * Command descriptor for materal:enum-apply
 */
export const enumApplyCommand = {
  name: "materal:enum-apply",
  summary: "Apply enum patch JSON and generate model files",
  description: "Applies an enum patch JSON file (created by materal:enum-plan) and generates TypeScript model files with enum types.",
  options: [
    {
      flags: "-o, --output <dir>",
      description: "Output directory for generated model files",
      required: true,
    },
    {
      flags: "-p, --patch <file>",
      description: "Enum patch JSON file path",
      required: true,
    },
    {
      flags: "-s, --style <declaration|module>",
      description: "Model output style (default: declaration)",
      defaultValue: "declaration",
    },
    {
      flags: "-c, --conflict-policy <openapi-first|patch-first|provider-first>",
      description: "Conflict policy for enum merge (default: patch-first)",
      defaultValue: "patch-first",
    },
    {
      flags: "-n, --name <schema>",
      description: "Generate specific schema names only; repeatable",
      required: false,
    },
  ],
  examples: [
    "aptx-ft materal enum-apply -o ./src/models -p enum-plan.json",
    "aptx-ft materal enum-apply -o ./models -p ./plans/enums.json --style module",
    "aptx-ft materal enum-apply -o ./src/models -p enum-plan.json --conflict-policy openapi-first",
  ],
  handler: runModelEnumApply,
};
