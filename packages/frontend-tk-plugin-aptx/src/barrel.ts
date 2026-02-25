import { CommandDescriptor, PluginContext } from '@aptx/frontend-tk-core';

/**
 * barrel:gen command - Generate barrel index.ts files for existing TypeScript files
 */
export const barrelCommand: CommandDescriptor = {
  name: 'barrel:gen',
  summary: 'Generate barrel index.ts files for existing TypeScript files',
  description: 'Scans a directory and generates barrel index.ts files for all subdirectories. Each subdirectory containing .ts files will get an index.ts that exports all files. The input directory will also get an index.ts that exports all subdirectories.',
  requiresOpenApi: false,
  options: [
    {
      flags: '-i, --input <path>',
      description: 'Input directory to scan for .ts files',
      required: true,
    },
  ],
  examples: [
    'aptx-ft barrel gen -i ./src/api',
    'aptx-ft barrel gen -i ./src/functions',
  ],
  handler: async (ctx: PluginContext, args: Record<string, unknown>) => {
    const { binding, log } = ctx;
    const input = args.input as string;

    if (!input) {
      throw new Error('--input is required');
    }

    log(`Generating barrel files for directory: ${input}`);
    // barrel:gen doesn't need an OpenAPI spec, so we don't pass input
    binding.runCli({
      input: undefined,
      command: 'barrel:gen',
      options: ['--input', input],
    });
    log('Barrel files generated successfully');
  },
};
