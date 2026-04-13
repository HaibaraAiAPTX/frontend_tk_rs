import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@aptx/frontend-tk-binding', () => ({
  runCli: vi.fn(),
  getIr: vi.fn(),
}));

import * as binding from '@aptx/frontend-tk-binding';
import { createCli } from '../src/cli';
import modelPlugin from '../../frontend-tk-plugin-model/src/index';

describe('model commands output short flag', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    process.exitCode = 0;
  });

  it('accepts -o for model ir and forwards output to binding', async () => {
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const cli = createCli();
    cli.use(modelPlugin);

    await cli.run([
      'node',
      'aptx-ft',
      '--input',
      'openapi.json',
      'model',
      'ir',
      '-o',
      './model-ir.json',
    ]);

    expect(binding.runCli).toHaveBeenCalledWith({
      input: 'openapi.json',
      command: 'model:ir',
      options: ['--output', './model-ir.json'],
      plugin: undefined,
    });
    expect(process.exitCode).toBe(0);

    errorSpy.mockRestore();
  });
});
