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
      options: [],
      handler: async (ctx, args) => {
        ctx.log('Hello from test plugin!');
      },
    },
  ],
};

module.exports = testPlugin;
module.exports.default = testPlugin;
