import { CommandDescriptor, PluginContext } from '@aptx/frontend-tk-core';

/**
 * python:tools command - Generate OpenAI function calling tools.json
 */
export const toolsCommand: CommandDescriptor = {
  name: 'python:tools',
  summary: 'Generate OpenAI function calling tools.json from OpenAPI specification',
  description:
    'Generates a tools.json file containing OpenAI function calling definitions based on the OpenAPI specification.',
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
    'aptx-ft python tools -i openapi.json -o ./tools',
    'aptx-ft python tools -i openapi.json -o ./tools --dry-run',
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

    log(`Generating Python tools from ${input} to ${output}`);
    binding.runCli({
      input,
      command: 'python:tools',
      options,
    });
    log('Python tools generated successfully');
  },
};
