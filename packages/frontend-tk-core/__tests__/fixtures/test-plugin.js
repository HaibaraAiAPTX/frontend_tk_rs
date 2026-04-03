/**
 * Test fixture: external JS plugin for full execution flow tests.
 * Handler captures ctx/args into __testPluginCalls for assertions.
 */

// Global capture store — tests read from here
globalThis.__testPluginCalls = [];

const testPlugin = {
  descriptor: {
    name: 'test-plugin',
    version: '1.0.0',
    namespaceDescription: 'Test plugin namespace',
  },
  commands: [
    {
      name: 'test:hello',
      summary: 'A test hello command',
      requiresOpenApi: false,
      options: [
        {
          flags: '-o, --output <path>',
          description: 'Output path',
        },
      ],
      handler: async (ctx, args) => {
        globalThis.__testPluginCalls.push({ ctx, args });
        ctx.log('Hello from test plugin!');
      },
    },
    {
      name: 'test:echo',
      summary: 'Echo command',
      requiresOpenApi: false,
      options: [],
      handler: async (ctx, args) => {
        globalThis.__testPluginCalls.push({ ctx, args });
        ctx.log(`Echo: ${JSON.stringify(args)}`);
      },
    },
  ],
  renderers: [
    {
      id: 'test-renderer',
      render: async (ctx, options) => {
        ctx.log('test-renderer executed');
      },
    },
  ],
};

module.exports = testPlugin;
module.exports.default = testPlugin;
