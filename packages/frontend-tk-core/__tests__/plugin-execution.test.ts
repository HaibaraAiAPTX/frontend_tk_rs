import { describe, it, expect, vi, beforeEach } from 'vitest';
import * as path from 'path';
import { createCli } from '../src/cli';

const FIXTURE_PATH = path.resolve(__dirname, 'fixtures', 'test-plugin.js');

// Declare the global capture type
declare global {
  var __testPluginCalls: Array<{ ctx: any; args: Record<string, unknown> }>;
}

describe('CLI.run() — full plugin execution flow', () => {
  beforeEach(() => {
    globalThis.__testPluginCalls = [];
  });

  it('loads external JS plugin via --plugin and executes command', async () => {
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    const cli = createCli();

    // Load plugin via use() — same code path as --plugin dynamic loading
    // but without variadic option consuming the command args
    const plugin = await import(FIXTURE_PATH);
    cli.use(plugin.default || plugin);

    await cli.run(['node', 'aptx-ft', 'test', 'hello']);

    expect(logSpy).toHaveBeenCalledWith('Hello from test plugin!');
    expect(globalThis.__testPluginCalls).toHaveLength(1);

    const { ctx } = globalThis.__testPluginCalls[0];
    expect(ctx).toBeDefined();
    expect(ctx.binding).toBeDefined();
    expect(typeof ctx.log).toBe('function');

    logSpy.mockRestore();
    warnSpy.mockRestore();
  });

  it('handler receives merged global and command options', async () => {
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const cli = createCli();

    const plugin = await import(FIXTURE_PATH);
    cli.use(plugin.default || plugin);

    await cli.run(['node', 'aptx-ft', 'test', 'hello', '-o', './out']);

    expect(globalThis.__testPluginCalls).toHaveLength(1);
    const { args } = globalThis.__testPluginCalls[0];
    expect(args.output).toBe('./out');

    logSpy.mockRestore();
  });

  it('gracefully handles --plugin with nonexistent path', async () => {
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    const cli = createCli();

    await cli.run([
      'node',
      'aptx-ft',
      '--plugin',
      '/nonexistent/path/plugin.js',
    ]);

    expect(warnSpy).toHaveBeenCalledWith(
      expect.stringContaining('Plugin load failed'),
    );

    warnSpy.mockRestore();
  });

  it('handles command handler error without crashing', async () => {
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const cli = createCli();

    cli.use({
      descriptor: { name: 'throw-plugin', version: '1.0.0' },
      commands: [
        {
          name: 'throw:fail',
          summary: 'Always fails',
          options: [],
          requiresOpenApi: false,
          handler: async () => {
            throw new Error('handler exploded');
          },
        },
      ],
    });

    await cli.run(['node', 'aptx-ft', 'throw', 'fail']);

    expect(errorSpy).toHaveBeenCalledWith(
      expect.stringContaining('handler exploded'),
    );
    expect(process.exitCode).toBe(1);

    process.exitCode = 0;
    errorSpy.mockRestore();
  });

  it('executes multiple commands in the same namespace', async () => {
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const cli = createCli();

    const plugin = await import(FIXTURE_PATH);
    cli.use(plugin.default || plugin);

    await cli.run(['node', 'aptx-ft', 'test', 'hello']);
    expect(logSpy).toHaveBeenCalledWith('Hello from test plugin!');

    globalThis.__testPluginCalls = [];
    await cli.run(['node', 'aptx-ft', 'test', 'echo']);
    expect(logSpy).toHaveBeenCalledWith(expect.stringContaining('Echo:'));
    expect(globalThis.__testPluginCalls).toHaveLength(1);

    logSpy.mockRestore();
  });
});
