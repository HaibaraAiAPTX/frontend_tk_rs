import { describe, it, expect, vi } from 'vitest';
import { createCli } from '../src/cli';
import type { Plugin, CommandDescriptor, RendererDescriptor } from '../src/types';

function makePlugin(overrides?: Partial<Plugin>): Plugin {
  return {
    descriptor: { name: 'mock-plugin', version: '1.0.0' },
    commands: [
      {
        name: 'mock:cmd',
        summary: 'A mock command',
        options: [],
        handler: vi.fn(),
      },
    ],
    ...overrides,
  };
}

describe('CLI.use() — plugin registration', () => {
  it('registers commands from a plugin', () => {
    const cmd: CommandDescriptor = {
      name: 'foo:bar',
      summary: 'Foo bar',
      options: [],
      handler: vi.fn(),
    };
    const cli = createCli();
    cli.use(makePlugin({ commands: [cmd] }));

    const registered = cli.getRegisteredCommands();
    expect(registered).toHaveLength(1);
    expect(registered[0].name).toBe('foo:bar');
  });

  it('creates namespace from colon-separated command name', () => {
    const cli = createCli();
    cli.use(
      makePlugin({
        descriptor: { name: 'ns-plugin', version: '1.0.0', namespaceDescription: 'NS desc' },
        commands: [
          { name: 'ns:one', summary: 'One', options: [], handler: vi.fn() },
          { name: 'ns:two', summary: 'Two', options: [], handler: vi.fn() },
        ],
      }),
    );

    const registered = cli.getRegisteredCommands();
    expect(registered).toHaveLength(2);
    expect(registered.map((c) => c.name)).toEqual(['ns:one', 'ns:two']);
  });

  it('registers renderers from a plugin', () => {
    const renderer: RendererDescriptor = {
      id: 'my-renderer',
      render: vi.fn(),
    };
    const cli = createCli();
    cli.use(makePlugin({ renderers: [renderer] }));

    expect(cli.findRenderer('my-renderer')).toBe(renderer);
  });

  it('returns undefined for unregistered renderer', () => {
    const cli = createCli();
    expect(cli.findRenderer('nonexistent')).toBeUndefined();
  });

  it('calls plugin init with context', async () => {
    const init = vi.fn();
    const cli = createCli();
    cli.use(makePlugin({ init }));

    // init is called via microtask — flush
    await new Promise((r) => setTimeout(r, 0));
    expect(init).toHaveBeenCalledTimes(1);
    expect(init).toHaveBeenCalledWith(
      expect.objectContaining({
        binding: expect.anything(),
        log: expect.any(Function),
      }),
    );
  });

  it('registers multiple plugins without conflict', () => {
    const cli = createCli();
    cli.use(
      makePlugin({
        descriptor: { name: 'plugin-a', version: '1.0.0' },
        commands: [
          { name: 'a:cmd', summary: 'A cmd', options: [], handler: vi.fn() },
        ],
        renderers: [{ id: 'renderer-a', render: vi.fn() }],
      }),
    );
    cli.use(
      makePlugin({
        descriptor: { name: 'plugin-b', version: '1.0.0' },
        commands: [
          { name: 'b:cmd', summary: 'B cmd', options: [], handler: vi.fn() },
        ],
        renderers: [{ id: 'renderer-b', render: vi.fn() }],
      }),
    );

    expect(cli.getRegisteredCommands()).toHaveLength(2);
    expect(cli.findRenderer('renderer-a')).toBeDefined();
    expect(cli.findRenderer('renderer-b')).toBeDefined();
  });

  it('returns a copy from getRegisteredCommands', () => {
    const cli = createCli();
    cli.use(makePlugin());

    const list = cli.getRegisteredCommands();
    list.push({
      name: 'injected',
      summary: 'Injected',
      options: [],
      handler: vi.fn(),
    });

    // Original should not be mutated
    expect(cli.getRegisteredCommands()).toHaveLength(1);
  });
});
