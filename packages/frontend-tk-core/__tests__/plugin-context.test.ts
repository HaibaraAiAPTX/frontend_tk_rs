import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { PluginContext } from '../src/types';

describe('PluginContext.getIr', () => {
  let ctx: PluginContext;

  beforeEach(() => {
    const binding = {
      getIr: vi.fn(),
      runCli: vi.fn(),
    } as any;
    ctx = {
      binding,
      log: vi.fn(),
      getIr(inputPath: string) {
        return binding.getIr(inputPath);
      },
    };
  });

  it('calls binding.getIr with the provided path', () => {
    const mockIr = {
      project: { package_name: 'test', terminals: [] },
      endpoints: [],
      model_import: null,
      client_import: null,
      output_root: null,
    };
    ctx.binding.getIr.mockReturnValue(mockIr);

    const result = ctx.getIr('/path/to/openapi.json');

    expect(ctx.binding.getIr).toHaveBeenCalledWith('/path/to/openapi.json');
    expect(result).toEqual(mockIr);
  });

  it('propagates errors from binding', () => {
    ctx.binding.getIr.mockImplementation(() => {
      throw new Error('Failed to read OpenAPI file: not found');
    });

    expect(() => ctx.getIr('/nonexistent.json')).toThrow('Failed to read OpenAPI file');
  });
});
