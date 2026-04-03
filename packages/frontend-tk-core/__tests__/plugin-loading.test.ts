import { describe, it, expect } from 'vitest';
import { isBinaryPlugin, resolvePluginPath } from '../src/cli';

describe('isBinaryPlugin', () => {
  it('identifies binary plugin extensions', () => {
    expect(isBinaryPlugin('foo.node')).toBe(true);
    expect(isBinaryPlugin('foo.dll')).toBe(true);
    expect(isBinaryPlugin('foo.so')).toBe(true);
    expect(isBinaryPlugin('foo.dylib')).toBe(true);
  });

  it('identifies JS plugins as non-binary', () => {
    expect(isBinaryPlugin('foo.js')).toBe(false);
    expect(isBinaryPlugin('foo.mjs')).toBe(false);
    expect(isBinaryPlugin('foo.cjs')).toBe(false);
  });
});

describe('resolvePluginPath', () => {
  it('resolves relative paths to absolute', () => {
    const resolved = resolvePluginPath('./my-plugin.js');
    expect(resolved).not.toContain('./');
    expect(resolved).toContain('my-plugin.js');
  });

  it('keeps absolute paths unchanged', () => {
    const abs = process.platform === 'win32'
      ? 'C:\\plugins\\test.js'
      : '/plugins/test.js';
    expect(resolvePluginPath(abs)).toBe(abs);
  });
});
