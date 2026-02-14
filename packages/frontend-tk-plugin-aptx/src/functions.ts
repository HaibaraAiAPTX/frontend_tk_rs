import { CommandDescriptor, PluginContext } from '@aptx/frontend-tk-core';

/**
 * aptx:functions command - Generate functions module code
 */
export const functionsCommand: CommandDescriptor = {
  name: 'aptx:functions',
  summary: 'Generate functions module from OpenAPI specification',
  description: 'Generates a functions module based on the OpenAPI specification. This provides a simple function-based API for making backend calls.',
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
      flags: '--client-mode <mode>',
      description: 'API client import mode: global (default) | local | package',
      defaultValue: 'global',
    },
    {
      flags: '--client-path <path>',
      description: 'Relative path to local client file (for --client-mode=local)',
    },
    {
      flags: '--client-package <name>',
      description: 'Package name for custom client (for --client-mode=package)',
    },
    {
      flags: '--client-import-name <name>',
      description: 'Custom import name (default: getApiClient)',
    },
    {
      flags: '--model-mode <mode>',
      description: 'Model import mode: relative (default) | package',
    },
    {
      flags: '--model-path <path>',
      description: 'Model import base path/package (e.g. ../../domains or @my-org/models)',
    },
  ],
  examples: [
    'aptx-ft aptx functions -i openapi.json -o ./generated',
    'aptx-ft aptx functions -i https://api.example.com/openapi.json -o ./src/api',
    'aptx-ft aptx functions -i openapi.json -o ./generated --client-mode local --client-path ./api/client.ts',
  ],
  handler: async (ctx: PluginContext, args: Record<string, unknown>) => {
    const { binding, log } = ctx;
    const input = args.input as string;
    const output = args.output as string;
    const clientMode = args.clientMode as string | undefined;
    const clientPath = args.clientPath as string | undefined;
    const clientPackage = args.clientPackage as string | undefined;
    const clientImportName = args.clientImportName as string | undefined;
    const modelMode = args.modelMode as string | undefined;
    const modelPath = args.modelPath as string | undefined;

    if (!input) {
      throw new Error('--input is required');
    }
    if (!output) {
      throw new Error('--output is required');
    }

    const options = ['--terminal', 'functions', '--output', output];

    // Add client import options if provided
    if (clientMode && clientMode !== 'global') {
      options.push('--client-mode', clientMode);
    }
    if (clientMode === 'local' && clientPath) {
      options.push('--client-path', clientPath);
    }
    if (clientMode === 'package' && clientPackage) {
      options.push('--client-package', clientPackage);
    }
    if (clientImportName) {
      options.push('--client-import-name', clientImportName);
    }
    if (modelMode) {
      options.push('--model-mode', modelMode);
    }
    if (modelPath) {
      options.push('--model-path', modelPath);
    }

    log(`Generating functions module from ${input} to ${output}`);
    binding.runCli({
      input,
      command: 'terminal:codegen',
      options,
    });
    log('Functions module generated successfully');
  },
};
