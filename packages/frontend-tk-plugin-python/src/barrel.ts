import { CommandDescriptor, PluginContext } from '@aptx/frontend-tk-core';

/**
 * python:barrel command - Generate Python package __init__.py files
 */
export const barrelCommand: CommandDescriptor = {
  name: 'python:barrel',
  summary: 'Generate Python package __init__.py files for existing Python files',
  description:
    'Scans a directory and generates Python package __init__.py files for generated modules. Each package directory gets an __init__.py that re-exports its child modules, and the input directory root also gets an __init__.py.',
  requiresOpenApi: false,
  options: [
    {
      flags: '-i, --input <path>',
      description: 'Input directory to scan for generated Python files',
      required: true,
    },
  ],
  examples: [
    'aptx-ft python barrel -i ./generated',
    'aptx-ft python barrel -i ./generated/functions',
  ],
  handler: async (ctx: PluginContext, args: Record<string, unknown>) => {
    const { binding, log } = ctx;
    const input = args.input as string;

    if (!input) {
      throw new Error('--input is required');
    }

    log(`Generating Python package __init__.py files for directory: ${input}`);
    binding.runCli({
      input: undefined,
      command: 'python:barrel',
      options: ['--input', input],
    });
    log('Python package __init__.py files generated successfully');
  },
};
