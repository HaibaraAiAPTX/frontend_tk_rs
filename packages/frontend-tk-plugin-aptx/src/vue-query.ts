import { CommandDescriptor, PluginContext } from '@aptx/frontend-tk-core';

/**
 * aptx:vue-query command - Generate Vue Query composables code
 */
export const vueQueryCommand: CommandDescriptor = {
  name: 'aptx:vue-query',
  summary: 'Generate Vue Query composables from OpenAPI specification',
  description: 'Generates Vue Query composables based on the OpenAPI specification. This provides Vue composables for data fetching and mutations using TanStack Query for Vue.',
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
  ],
  examples: [
    'aptx-ft aptx vue-query -i openapi.json -o ./generated',
    'aptx-ft aptx vue-query -i https://api.example.com/openapi.json -o ./src/api',
    'aptx-ft aptx vue-query -i openapi.json -o ./generated --client-mode local --client-path ./api/client.ts',
  ],
  handler: async (ctx: PluginContext, args: Record<string, unknown>) => {
    const { binding, log } = ctx;
    const input = args.input as string;
    const output = args.output as string;
    const clientMode = args.clientMode as string | undefined;
    const clientPath = args.clientPath as string | undefined;
    const clientPackage = args.clientPackage as string | undefined;
    const clientImportName = args.clientImportName as string | undefined;

    if (!input) {
      throw new Error('--input is required');
    }
    if (!output) {
      throw new Error('--output is required');
    }

    const options = ['--terminal', 'vue-query', '--output', output];

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

    log(`Generating Vue Query composables from ${input} to ${output}`);
    binding.runCli({
      input,
      command: 'terminal:codegen',
      options,
    });
    log('Vue Query composables generated successfully');
  },
};
