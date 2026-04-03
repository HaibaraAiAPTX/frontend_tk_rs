import { describe, it, expect, vi, beforeEach } from 'vitest';
import * as path from 'path';
import { createCli } from '../src/cli';

const FIXTURE_PATH = path.resolve(__dirname, 'fixtures', 'test-plugin.js');

// Declare the global capture type
declare global {
  var __testPluginCalls: Array<{ ctx: any; args: Record<string, unknown> }>;
}

describe('CLI --no-manifest negation flag', () => {
  beforeEach(() => {
    globalThis.__testPluginCalls = [];
  });

  it('--no-manifest sets args.manifest to false (not args.noManifest)', async () => {
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const cli = createCli();

    const plugin = await import(FIXTURE_PATH);
    cli.use(plugin.default || plugin);

    await cli.run(['node', 'aptx-ft', 'test', 'no-manifest', '--no-manifest']);

    expect(globalThis.__testPluginCalls).toHaveLength(1);
    const { args } = globalThis.__testPluginCalls[0];

    // Commander.js negation: --no-manifest sets `manifest` to false
    expect(args.manifest).toBe(false);
    // args.noManifest should NOT exist (this was the bug)
    expect((args as any).noManifest).toBeUndefined();

    logSpy.mockRestore();
  });

  it('without --no-manifest, args.manifest defaults to true', async () => {
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const cli = createCli();

    const plugin = await import(FIXTURE_PATH);
    cli.use(plugin.default || plugin);

    await cli.run(['node', 'aptx-ft', 'test', 'no-manifest']);

    expect(globalThis.__testPluginCalls).toHaveLength(1);
    const { args } = globalThis.__testPluginCalls[0];

    // Without --no-manifest, Commander.js sets `manifest` to true (negation default)
    expect(args.manifest).toBe(true);

    logSpy.mockRestore();
  });

  it('handler correctly derives noManifest from args.manifest === false', async () => {
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const cli = createCli();

    const plugin = await import(FIXTURE_PATH);
    cli.use(plugin.default || plugin);

    // With --no-manifest
    await cli.run(['node', 'aptx-ft', 'test', 'no-manifest', '--no-manifest']);
    expect(logSpy).toHaveBeenCalledWith(
      expect.stringContaining('noManifest=true'),
    );
    expect(logSpy).toHaveBeenCalledWith(
      expect.stringContaining('--no-manifest'),
    );

    globalThis.__testPluginCalls = [];
    vi.clearAllMocks();

    // Without --no-manifest
    await cli.run(['node', 'aptx-ft', 'test', 'no-manifest']);
    expect(logSpy).toHaveBeenCalledWith(
      expect.stringContaining('noManifest=false'),
    );

    logSpy.mockRestore();
  });
});
