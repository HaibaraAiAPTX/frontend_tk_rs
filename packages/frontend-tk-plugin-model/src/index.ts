/**
 * @aptx/frontend-tk-plugin-model
 *
 * Model generation commands for the frontend toolkit CLI.
 * Provides commands like model:gen, model:ir, model:enum-plan, model:enum-apply.
 */

import type {
  Plugin,
  PluginContext,
  CommandDescriptor,
  OptionDescriptor,
} from '@aptx/frontend-tk-core';

/**
 * Common model command options
 */
const commonModelOptions: OptionDescriptor[] = [
  {
    flags: '-i, --input <path>',
    description: 'OpenAPI spec file path or URL',
    required: false,
  },
  {
    flags: '--output <path>',
    description: 'Output directory or file path',
    required: false,
  },
];

/**
 * Create model generation commands
 */
function createModelCommands(): CommandDescriptor[] {
  return [
    {
      name: 'model:gen',
      summary: 'Generate TypeScript model declarations from OpenAPI schemas',
      description: 'Generates TypeScript type definitions from OpenAPI schema definitions.',
      options: [
        ...commonModelOptions,
        {
          flags: '--style <declaration|module>',
          description: 'Model output style',
          defaultValue: 'module',
        },
        {
          flags: '--name <schema>',
          description: 'Generate specific schema names only; repeatable',
          required: false,
        },
      ],
      examples: [
        'aptx-ft model gen --input openapi.json --output ./src/models',
        'aptx-ft model gen --input openapi.json --output ./src/models --style module',
      ],
      handler: async (ctx: PluginContext, args: Record<string, unknown>) => {
        const binding = ctx.binding as any;
        if (typeof binding.runCli === 'function') {
          const options: string[] = [];
          if (args.output) options.push('--output', String(args.output));
          if (args.style) options.push('--style', String(args.style));
          if (args.name) {
            const names = Array.isArray(args.name) ? args.name : [args.name];
            names.forEach((n: string) => options.push('--name', n));
          }

          binding.runCli({
            input: args.input as string | undefined,
            command: 'model:gen',
            options,
            plugin: args.plugin as string[] | undefined,
          });
        } else {
          ctx.log('Native binding runCli not available for command: model:gen');
        }
      },
    },
    {
      name: 'model:ir',
      summary: 'Export model intermediate representation JSON',
      description: 'Exports the internal IR representation of OpenAPI schemas.',
      options: [
        ...commonModelOptions,
      ],
      examples: [
        'aptx-ft model ir --input openapi.json --output ./model-ir.json',
      ],
      handler: async (ctx: PluginContext, args: Record<string, unknown>) => {
        const binding = ctx.binding as any;
        if (typeof binding.runCli === 'function') {
          const options: string[] = [];
          if (args.output) options.push('--output', String(args.output));

          binding.runCli({
            input: args.input as string | undefined,
            command: 'model:ir',
            options,
            plugin: args.plugin as string[] | undefined,
          });
        } else {
          ctx.log('Native binding runCli not available for command: model:ir');
        }
      },
    },
    {
      name: 'model:enum-plan',
      summary: 'Export enum enrichment plan JSON',
      description: 'Analyzes enums in the OpenAPI spec and creates an enrichment plan.',
      options: [
        ...commonModelOptions,
        {
          flags: '--model-output <dir>',
          description: 'Existing generated model directory for enum name reuse',
          required: false,
        },
      ],
      examples: [
        'aptx-ft model enum-plan --input openapi.json --output ./enum-plan.json',
        'aptx-ft model enum-plan --input openapi.json --output ./tmp/enum-plan.json --model-output ./src/models',
      ],
      handler: async (ctx: PluginContext, args: Record<string, unknown>) => {
        const binding = ctx.binding as any;
        if (typeof binding.runCli === 'function') {
          const options: string[] = [];
          if (args.output) options.push('--output', String(args.output));
          if (args.modelOutput) options.push('--model-output', String(args.modelOutput));

          binding.runCli({
            input: args.input as string | undefined,
            command: 'model:enum-plan',
            options,
            plugin: args.plugin as string[] | undefined,
          });
        } else {
          ctx.log('Native binding runCli not available for command: model:enum-plan');
        }
      },
    },
    {
      name: 'model:enum-apply',
      summary: 'Apply enum patch and generate models',
      description: 'Applies an enum enrichment patch and generates enhanced model files.',
      options: [
        ...commonModelOptions,
        {
          flags: '--patch <file>',
          description: 'Enum patch JSON file path',
          required: false,
        },
        {
          flags: '--style <declaration|module>',
          description: 'Model output style',
          defaultValue: 'declaration',
        },
        {
          flags: '--conflict-policy <openapi-first|patch-first|provider-first>',
          description: 'Conflict policy for enum merge',
          defaultValue: 'patch-first',
        },
        {
          flags: '--name <schema>',
          description: 'Generate specific schema names only; repeatable',
          required: false,
        },
      ],
      examples: [
        'aptx-ft model enum-apply --input openapi.json --patch ./enum-plan.json --output ./src/models',
      ],
      handler: async (ctx: PluginContext, args: Record<string, unknown>) => {
        const binding = ctx.binding as any;
        if (typeof binding.runCli === 'function') {
          const options: string[] = [];
          if (args.output) options.push('--output', String(args.output));
          if (args.patch) options.push('--patch', String(args.patch));
          if (args.style) options.push('--style', String(args.style));
          if (args.conflictPolicy) options.push('--conflict-policy', String(args.conflictPolicy));
          if (args.name) {
            const names = Array.isArray(args.name) ? args.name : [args.name];
            names.forEach((n: string) => options.push('--name', n));
          }

          binding.runCli({
            input: args.input as string | undefined,
            command: 'model:enum-apply',
            options,
            plugin: args.plugin as string[] | undefined,
          });
        } else {
          ctx.log('Native binding runCli not available for command: model:enum-apply');
        }
      },
    },
  ];
}

/**
 * Model plugin descriptor
 */
const modelPlugin: Plugin = {
  descriptor: {
    name: '@aptx/frontend-tk-plugin-model',
    version: '0.1.0',
    namespaceDescription: 'Model generation',
  },
  commands: createModelCommands(),
  renderers: [],
};

export default modelPlugin;
