/**
 * @aptx/frontend-tk-plugin-aptx
 *
 * Aptx namespace commands for the frontend toolkit CLI.
 * Provides commands like aptx:functions, aptx:react-query, aptx:vue-query.
 */

import type {
  Plugin,
  PluginContext,
  CommandDescriptor,
  RendererDescriptor,
} from '@aptx/frontend-tk-core';

/**
 * Create aptx namespace commands that delegate to the native binding
 */
function createAptxCommands(): CommandDescriptor[] {
  const commandNames = [
    'aptx:functions',
    'aptx:react-query',
    'aptx:vue-query',
  ];

  return commandNames.map((name) => ({
    name,
    summary: `Generate ${name.replace('aptx:', '')} code from OpenAPI spec`,
    description: `Native command for generating ${name} frontend code.`,
    options: [],
    // Convert namespace:command to "namespace command" for CLI usage
    examples: [`aptx-ft ${name.replace(':', ' ')} --input openapi.json --output ./src`],
    handler: async (ctx: PluginContext, args: Record<string, unknown>) => {
      // Delegate to the native binding's runCli function
      const binding = ctx.binding as any;
      if (typeof binding.runCli === 'function') {
        const options = Object.entries(args)
          .flatMap(([key, value]) => {
            if (value === undefined || value === null) return [];
            const flag = key.length === 1 ? `-${key}` : `--${key}`;
            return Array.isArray(value) ? value.map((v) => [flag, String(v)]) : [[flag, String(value)]];
          })
          .flat();

        binding.runCli({
          input: args.input as string | undefined,
          command: name,
          options,
          plugin: args.plugin as string[] | undefined,
        });
      } else {
        ctx.log(`Native binding runCli not available for command: ${name}`);
      }
    },
  }));
}

/**
 * Aptx plugin descriptor
 */
const aptxPlugin: Plugin = {
  descriptor: {
    name: '@aptx/frontend-tk-plugin-aptx',
    version: '0.1.0',
    namespaceDescription: '@aptx ecosystem (functions, react-query, vue-query)',
  },
  commands: createAptxCommands(),
  renderers: [],
};

export default aptxPlugin;
