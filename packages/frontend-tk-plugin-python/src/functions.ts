import { CommandDescriptor, PluginContext } from '@aptx/frontend-tk-core';

/**
 * python:functions command - Generate Python functions module code
 */
export const functionsCommand: CommandDescriptor = {
  name: 'python:functions',
  summary: 'Generate Python functions module from OpenAPI specification',
  description:
    'Generates Python functions module based on the OpenAPI specification.',
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
      flags: '--model-mode <mode>',
      description: 'Model import mode: relative (default) | package',
    },
    {
      flags: '--model-path <path>',
      description:
        'Model import base path/package (e.g. ../../domains or @my-org/models)',
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
    'aptx-ft python functions -i openapi.json -o ./generated',
    'aptx-ft python functions -i openapi.json -o ./generated --dry-run',
  ],
  handler: async (ctx: PluginContext, args: Record<string, unknown>) => {
    const { binding, log } = ctx;
    const input = args.input as string;
    const output = args.output as string;
    const modelMode = args.modelMode as string | undefined;
    const modelPath = args.modelPath as string | undefined;
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

    if (modelMode) {
      options.push('--model-mode', modelMode);
    }
    if (modelPath) {
      options.push('--model-path', modelPath);
    }
    if (noManifest) {
      options.push('--no-manifest');
    }
    if (manifestDir) {
      options.push('--manifest-dir', manifestDir);
    }
    if (dryRun) {
      options.push('--dry-run');
    }

    log(`Generating Python functions from ${input} to ${output}`);
    binding.runCli({
      input,
      command: 'python:functions',
      options,
    });
    log('Python functions generated successfully');
  },
};
