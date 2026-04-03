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
    {
      name: 'test:no-manifest',
      summary: 'Test --no-manifest negation flag',
      requiresOpenApi: false,
      options: [
        {
          flags: '--no-manifest',
          description: 'Disable manifest tracking',
          defaultValue: true,
        },
        {
          flags: '--manifest-dir <path>',
          description: 'Manifest directory',
          defaultValue: '.generated',
        },
      ],
      handler: async (ctx, args) => {
        globalThis.__testPluginCalls.push({ ctx, args });
        // Commander.js treats --no-manifest as negation: sets `manifest` property
        // When --no-manifest passed: manifest=false; default: manifest=true
        const noManifest = (args.manifest) === false;
        const passedOptions = [];
        if (noManifest) passedOptions.push('--no-manifest');
        if (args.manifestDir) passedOptions.push('--manifest-dir', String(args.manifestDir));
        ctx.log(`noManifest=${noManifest}, options=${JSON.stringify(passedOptions)}`);
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
