/**
 * @aptx/frontend-tk-plugin-standard
 *
 * Standard namespace commands for the frontend toolkit CLI.
 * Provides commands like std:axios-ts, std:axios-js, std:uniapp.
 */

import type {
  Plugin,
  PluginContext,
  CommandDescriptor,
  RendererDescriptor,
} from '@aptx/frontend-tk-core';

/**
 * Create standard namespace commands that delegate to the native binding
 */
function createStandardCommands(): CommandDescriptor[] {
  const commandNames = [
    'std:axios-ts',
    'std:axios-js',
    'std:uniapp',
  ];

  return commandNames.map((name) => ({
    name,
    summary: `Generate ${name.replace('std:', '')} code from OpenAPI spec`,
    description: `Native command for generating ${name} frontend code.`,
    options: [],
    examples: [`aptx-ft ${name} --input openapi.json --output ./src`],
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
 * Standard plugin descriptor
 */
const standardPlugin: Plugin = {
  descriptor: {
    name: '@aptx/frontend-tk-plugin-standard',
    version: '0.1.0',
    namespaceDescription: 'Standard library (axios-ts, axios-js, uniapp)',
  },
  commands: createStandardCommands(),
  renderers: [],
};

export default standardPlugin;
