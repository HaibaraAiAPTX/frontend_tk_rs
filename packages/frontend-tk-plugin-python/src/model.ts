import { CommandDescriptor, PluginContext } from '@aptx/frontend-tk-core';

/**
 * python:model command - Generate Python Pydantic models from OpenAPI specification
 */
export const modelCommand: CommandDescriptor = {
  name: 'python:model',
  summary: 'Generate Python Pydantic models from OpenAPI specification',
  description:
    'Generates Pydantic model classes based on the OpenAPI specification.',
  options: [
    {
      flags: '-i, --input <path>',
      description: 'OpenAPI specification file path or URL',
      required: true,
    },
    {
      flags: '-o, --output <dir>',
      description: 'Output directory for generated files',
      required: true,
    },
    {
      flags: '--no-manifest',
      description:
        'Disable manifest tracking and deletion report generation',
      defaultValue: true,
    },
    {
      flags: '--manifest-dir <path>',
      description:
        'Custom directory for manifest files (default: .generated)',
      defaultValue: '.generated',
    },
    {
      flags: '--dry-run',
      description:
        'Preview mode: generate deletion report without updating manifest',
      defaultValue: false,
    },
  ],
  examples: [
    'aptx-ft python model -i openapi.json -o ./models',
    'aptx-ft python model -i openapi.json -o ./models --dry-run',
  ],
  handler: async (ctx: PluginContext, args: Record<string, unknown>) => {
    const { binding, log } = ctx;
    const input = args.input as string;
    const output = args.output as string;
    const noManifest = (args.manifest as boolean | undefined) === false;
    const manifestDir = args.manifestDir as string | undefined;
    const dryRun = args.dryRun as boolean | undefined;

    if (!input) {
      throw new Error('--input is required');
    }
    if (!output) {
      throw new Error('--output is required');
    }

    const options = ['--output', output];

    if (noManifest) {
      options.push('--no-manifest');
    }
    if (manifestDir) {
      options.push('--manifest-dir', manifestDir);
    }
    if (dryRun) {
      options.push('--dry-run');
    }

    log(`Generating Python models from ${input} to ${output}`);
    binding.runCli({
      input,
      command: 'python:model',
      options,
    });
    log('Python models generated successfully');
  },
};
