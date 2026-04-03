import { describe, it, expect, vi, beforeEach, afterAll } from 'vitest';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import { createCli } from '../src/cli';

describe('Runtime plugin registration via --plugin flag', () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'aptx-plugin-'));

  afterAll(() => {
    fs.rmSync(tempDir, { recursive: true, force: true });
  });

  beforeEach(() => {
    vi.restoreAllMocks();
    globalThis.__initPluginFired = undefined;
    globalThis.__execPluginCalls = [];
    globalThis.__paramPluginCalls = [];
    globalThis.__ctxPluginCalls = [];
    globalThis.__defaultsPluginCalls = [];
  });

  it('run() loads plugin via --plugin and registers commands', async () => {
    const pluginPath = path.join(tempDir, 'auto-plugin.js');

    fs.writeFileSync(
      pluginPath,
      `module.exports = {
        descriptor: { name: 'auto-plugin', version: '1.0.0', namespaceDescription: 'Auto-loaded plugin' },
        commands: [{
          name: 'auto:run',
          summary: 'Auto-loaded run command',
          requiresOpenApi: false,
          options: [],
          handler: async () => {}
        }, {
          name: 'auto:build',
          summary: 'Auto-loaded build command',
          requiresOpenApi: false,
          options: [],
          handler: async () => {}
        }]
      };`,
      'utf-8',
    );

    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    const cli = createCli();
    await cli.run([
      'node', 'aptx-ft',
      '--plugin', pluginPath,
    ]);

    const commands = cli.getRegisteredCommands();
    expect(commands).toHaveLength(2);
    expect(commands.map((c) => c.name)).toContain('auto:run');
    expect(commands.map((c) => c.name)).toContain('auto:build');

    warnSpy.mockRestore();
  });

  it('run() loads multiple plugins via repeated --plugin flags', async () => {
    const pluginA = path.join(tempDir, 'multi-a.js');
    const pluginB = path.join(tempDir, 'multi-b.js');

    fs.writeFileSync(
      pluginA,
      `module.exports = {
        descriptor: { name: 'plugin-a', version: '1.0.0' },
        commands: [{
          name: 'alpha:cmd',
          summary: 'Alpha command',
          requiresOpenApi: false,
          options: [],
          handler: async () => {}
        }]
      };`,
      'utf-8',
    );

    fs.writeFileSync(
      pluginB,
      `module.exports = {
        descriptor: { name: 'plugin-b', version: '1.0.0' },
        commands: [{
          name: 'beta:cmd',
          summary: 'Beta command',
          requiresOpenApi: false,
          options: [],
          handler: async () => {}
        }]
      };`,
      'utf-8',
    );

    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    const cli = createCli();
    await cli.run([
      'node', 'aptx-ft',
      '--plugin', pluginA,
      '--plugin', pluginB,
    ]);

    const commands = cli.getRegisteredCommands();
    expect(commands).toHaveLength(2);
    expect(commands.map((c) => c.name)).toContain('alpha:cmd');
    expect(commands.map((c) => c.name)).toContain('beta:cmd');

    warnSpy.mockRestore();
  });

  it('run() fires plugin init hook during --plugin loading', async () => {
    const pluginPath = path.join(tempDir, 'init-plugin.js');

    fs.writeFileSync(
      pluginPath,
      `
      globalThis.__initPluginFired = false;
      module.exports = {
        descriptor: { name: 'init-plugin', version: '1.0.0' },
        commands: [{
          name: 'init:check',
          summary: 'Check init',
          requiresOpenApi: false,
          options: [],
          handler: async (ctx) => {
            ctx.log('initFired=' + globalThis.__initPluginFired);
          }
        }],
        init: async (ctx) => {
          globalThis.__initPluginFired = true;
          ctx.log('INIT_HOOK_FIRED');
        }
      };
      `,
      'utf-8',
    );

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    const cli = createCli();
    await cli.run([
      'node', 'aptx-ft',
      '--plugin', pluginPath,
      'init', 'check',
    ]);

    await new Promise((r) => setTimeout(r, 50));

    expect(globalThis.__initPluginFired).toBe(true);
    expect(logSpy).toHaveBeenCalledWith('INIT_HOOK_FIRED');
    expect(logSpy).toHaveBeenCalledWith('initFired=true');

    logSpy.mockRestore();
    warnSpy.mockRestore();
  });

  it('run() gracefully handles nonexistent plugin path via --plugin', async () => {
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    const cli = createCli();
    await cli.run([
      'node', 'aptx-ft',
      '--plugin', '/nonexistent/plugin.js',
    ]);

    expect(warnSpy).toHaveBeenCalledWith(
      expect.stringContaining('Plugin load failed'),
    );

    warnSpy.mockRestore();
  });

  it('run() skips binary plugin files via --plugin', async () => {
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    const fakeNode = path.join(tempDir, 'fake.node');
    fs.writeFileSync(fakeNode, 'binary content', 'utf-8');

    const cli = createCli();
    await cli.run([
      'node', 'aptx-ft',
      '--plugin', fakeNode,
    ]);

    expect(warnSpy).toHaveBeenCalledWith(
      expect.stringContaining('Binary plugin skipped'),
    );

    warnSpy.mockRestore();
  });

  it('run() executes command from plugin loaded via --plugin', async () => {
    const pluginPath = path.join(tempDir, 'exec-plugin.js');

    fs.writeFileSync(
      pluginPath,
      `
      globalThis.__execPluginCalls = globalThis.__execPluginCalls || [];
      module.exports = {
        descriptor: { name: 'exec-plugin', version: '1.0.0' },
        commands: [{
          name: 'exec:test',
          summary: 'Execution test',
          requiresOpenApi: false,
          options: [{ flags: '--msg <text>', description: 'Message', defaultValue: 'default' }],
          handler: async (ctx, args) => {
            globalThis.__execPluginCalls.push({ args });
            ctx.log('EXEC: ' + args.msg);
          }
        }]
      };
      `,
      'utf-8',
    );

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    const cli = createCli();
    await cli.run([
      'node', 'aptx-ft',
      '--plugin', pluginPath,
      'exec', 'test',
      '--msg', 'hello-runtime',
    ]);

    await new Promise((r) => setTimeout(r, 50));

    expect(globalThis.__execPluginCalls).toHaveLength(1);
    expect(globalThis.__execPluginCalls[0].args.msg).toBe('hello-runtime');
    expect(logSpy).toHaveBeenCalledWith('EXEC: hello-runtime');

    logSpy.mockRestore();
    warnSpy.mockRestore();
  });

  it('run() executes command and receives global options + command options', async () => {
    const pluginPath = path.join(tempDir, 'param-plugin.js');

    fs.writeFileSync(
      pluginPath,
      `
      globalThis.__paramPluginCalls = globalThis.__paramPluginCalls || [];
      module.exports = {
        descriptor: { name: 'param-plugin', version: '1.0.0' },
        commands: [{
          name: 'param:run',
          summary: 'Parameter test',
          requiresOpenApi: false,
          options: [
            { flags: '--format <type>', description: 'Output format', defaultValue: 'json' },
            { flags: '--verbose', description: 'Verbose mode', defaultValue: false }
          ],
          handler: async (ctx, args) => {
            globalThis.__paramPluginCalls.push({ ctx, args });
            ctx.log('FORMAT=' + args.format);
            ctx.log('VERBOSE=' + args.verbose);
            ctx.log('INPUT=' + (args.input || 'none'));
          }
        }]
      };
      `,
      'utf-8',
    );

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    const cli = createCli();
    await cli.run([
      'node', 'aptx-ft',
      '--plugin', pluginPath,
      '-i', './openapi.json',
      'param', 'run',
      '--format', 'yaml',
      '--verbose',
    ]);

    await new Promise((r) => setTimeout(r, 50));

    expect(globalThis.__paramPluginCalls).toHaveLength(1);
    const { args } = globalThis.__paramPluginCalls[0];
    expect(args.format).toBe('yaml');
    expect(args.verbose).toBe(true);
    expect(args.input).toBe('./openapi.json');

    expect(logSpy).toHaveBeenCalledWith('FORMAT=yaml');
    expect(logSpy).toHaveBeenCalledWith('VERBOSE=true');
    expect(logSpy).toHaveBeenCalledWith('INPUT=./openapi.json');

    logSpy.mockRestore();
    warnSpy.mockRestore();
  });

  it('run() command handler receives ctx with getIr and log', async () => {
    const pluginPath = path.join(tempDir, 'ctx-plugin.js');

    fs.writeFileSync(
      pluginPath,
      `
      globalThis.__ctxPluginCalls = globalThis.__ctxPluginCalls || [];
      module.exports = {
        descriptor: { name: 'ctx-plugin', version: '1.0.0' },
        commands: [{
          name: 'ctx:inspect',
          summary: 'Inspect context',
          requiresOpenApi: false,
          options: [],
          handler: async (ctx, args) => {
            globalThis.__ctxPluginCalls.push({
              hasLog: typeof ctx.log === 'function',
              hasBinding: typeof ctx.binding === 'object',
              hasGetIr: typeof ctx.getIr === 'function',
            });
            ctx.log('CTX_OK');
          }
        }]
      };
      `,
      'utf-8',
    );

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    const cli = createCli();
    await cli.run([
      'node', 'aptx-ft',
      '--plugin', pluginPath,
      'ctx', 'inspect',
    ]);

    await new Promise((r) => setTimeout(r, 50));

    expect(globalThis.__ctxPluginCalls).toHaveLength(1);
    const ctxInfo = globalThis.__ctxPluginCalls[0];
    expect(ctxInfo.hasLog).toBe(true);
    expect(ctxInfo.hasBinding).toBe(true);
    expect(ctxInfo.hasGetIr).toBe(true);
    expect(logSpy).toHaveBeenCalledWith('CTX_OK');

    logSpy.mockRestore();
    warnSpy.mockRestore();
  });

  it('run() command uses default values when options not provided', async () => {
    const pluginPath = path.join(tempDir, 'defaults-plugin.js');

    fs.writeFileSync(
      pluginPath,
      `
      globalThis.__defaultsPluginCalls = globalThis.__defaultsPluginCalls || [];
      module.exports = {
        descriptor: { name: 'defaults-plugin', version: '1.0.0' },
        commands: [{
          name: 'def:run',
          summary: 'Defaults test',
          requiresOpenApi: false,
          options: [
            { flags: '--count <n>', description: 'Count', defaultValue: 42 },
            { flags: '--silent', description: 'Silent mode', defaultValue: false }
          ],
          handler: async (ctx, args) => {
            globalThis.__defaultsPluginCalls.push({ args });
            ctx.log('COUNT=' + args.count);
            ctx.log('SILENT=' + args.silent);
          }
        }]
      };
      `,
      'utf-8',
    );

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    const cli = createCli();
    await cli.run([
      'node', 'aptx-ft',
      '--plugin', pluginPath,
      'def', 'run',
    ]);

    await new Promise((r) => setTimeout(r, 50));

    expect(globalThis.__defaultsPluginCalls).toHaveLength(1);
    const { args } = globalThis.__defaultsPluginCalls[0];
    expect(args.count).toBe(42);
    expect(args.silent).toBe(false);

    expect(logSpy).toHaveBeenCalledWith('COUNT=42');
    expect(logSpy).toHaveBeenCalledWith('SILENT=false');

    logSpy.mockRestore();
    warnSpy.mockRestore();
  });

  it('run() executes command and receives global options + command options', async () => {
    const pluginPath = path.join(tempDir, 'param-plugin.js');

    fs.writeFileSync(
      pluginPath,
      `
      globalThis.__paramPluginCalls = globalThis.__paramPluginCalls || [];
      module.exports = {
        descriptor: { name: 'param-plugin', version: '1.0.0' },
        commands: [{
          name: 'param:run',
          summary: 'Parameter test',
          requiresOpenApi: false,
          options: [
            { flags: '--format <type>', description: 'Output format', defaultValue: 'json' },
            { flags: '--verbose', description: 'Verbose mode', defaultValue: false }
          ],
          handler: async (ctx, args) => {
            globalThis.__paramPluginCalls.push({ ctx, args });
            ctx.log('FORMAT=' + args.format);
            ctx.log('VERBOSE=' + args.verbose);
            ctx.log('INPUT=' + (args.input || 'none'));
          }
        }]
      };
      `,
      'utf-8',
    );

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    const cli = createCli();
    await cli.run([
      'node', 'aptx-ft',
      '--plugin', pluginPath,
      '-i', './openapi.json',
      'param', 'run',
      '--format', 'yaml',
      '--verbose',
    ]);

    await new Promise((r) => setTimeout(r, 50));

    expect(globalThis.__paramPluginCalls).toHaveLength(1);
    const { args } = globalThis.__paramPluginCalls[0];
    expect(args.format).toBe('yaml');
    expect(args.verbose).toBe(true);
    expect(args.input).toBe('./openapi.json');

    expect(logSpy).toHaveBeenCalledWith('FORMAT=yaml');
    expect(logSpy).toHaveBeenCalledWith('VERBOSE=true');
    expect(logSpy).toHaveBeenCalledWith('INPUT=./openapi.json');

    logSpy.mockRestore();
    warnSpy.mockRestore();
  });

  it('run() command handler receives ctx with getIr and log', async () => {
    const pluginPath = path.join(tempDir, 'ctx-plugin.js');

    fs.writeFileSync(
      pluginPath,
      `
      globalThis.__ctxPluginCalls = globalThis.__ctxPluginCalls || [];
      module.exports = {
        descriptor: { name: 'ctx-plugin', version: '1.0.0' },
        commands: [{
          name: 'ctx:inspect',
          summary: 'Inspect context',
          requiresOpenApi: false,
          options: [],
          handler: async (ctx, args) => {
            globalThis.__ctxPluginCalls.push({
              hasLog: typeof ctx.log === 'function',
              hasBinding: typeof ctx.binding === 'object',
              hasGetIr: typeof ctx.getIr === 'function',
            });
            ctx.log('CTX_OK');
          }
        }]
      };
      `,
      'utf-8',
    );

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    const cli = createCli();
    await cli.run([
      'node', 'aptx-ft',
      '--plugin', pluginPath,
      'ctx', 'inspect',
    ]);

    await new Promise((r) => setTimeout(r, 50));

    expect(globalThis.__ctxPluginCalls).toHaveLength(1);
    const ctxInfo = globalThis.__ctxPluginCalls[0];
    expect(ctxInfo.hasLog).toBe(true);
    expect(ctxInfo.hasBinding).toBe(true);
    expect(ctxInfo.hasGetIr).toBe(true);
    expect(logSpy).toHaveBeenCalledWith('CTX_OK');

    logSpy.mockRestore();
    warnSpy.mockRestore();
  });

  it('run() command uses default values when options not provided', async () => {
    const pluginPath = path.join(tempDir, 'defaults-plugin.js');

    fs.writeFileSync(
      pluginPath,
      `
      globalThis.__defaultsPluginCalls = globalThis.__defaultsPluginCalls || [];
      module.exports = {
        descriptor: { name: 'defaults-plugin', version: '1.0.0' },
        commands: [{
          name: 'def:run',
          summary: 'Defaults test',
          requiresOpenApi: false,
          options: [
            { flags: '--count <n>', description: 'Count', defaultValue: 42 },
            { flags: '--silent', description: 'Silent mode', defaultValue: false }
          ],
          handler: async (ctx, args) => {
            globalThis.__defaultsPluginCalls.push({ args });
            ctx.log('COUNT=' + args.count);
            ctx.log('SILENT=' + args.silent);
          }
        }]
      };
      `,
      'utf-8',
    );

    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    const cli = createCli();
    await cli.run([
      'node', 'aptx-ft',
      '--plugin', pluginPath,
      'def', 'run',
    ]);

    await new Promise((r) => setTimeout(r, 50));

    expect(globalThis.__defaultsPluginCalls).toHaveLength(1);
    const { args } = globalThis.__defaultsPluginCalls[0];
    expect(args.count).toBe(42);
    expect(args.silent).toBe(false);

    expect(logSpy).toHaveBeenCalledWith('COUNT=42');
    expect(logSpy).toHaveBeenCalledWith('SILENT=false');

    logSpy.mockRestore();
    warnSpy.mockRestore();
  });
});

declare global {
  var __initPluginFired: boolean | undefined;
  var __execPluginCalls: Array<{ args: Record<string, unknown> }> | undefined;
  var __paramPluginCalls: Array<{ ctx: unknown; args: Record<string, unknown> }> | undefined;
  var __ctxPluginCalls: Array<{ hasLog: boolean; hasBinding: boolean; hasGetIr: boolean }> | undefined;
  var __defaultsPluginCalls: Array<{ args: Record<string, unknown> }> | undefined;
}
