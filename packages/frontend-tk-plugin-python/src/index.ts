/**
 * @aptx/frontend-tk-plugin-python
 *
 * Python codegen commands for the frontend toolkit CLI.
 * Provides commands like python:functions, python:model, python:tools.
 */

import type { Plugin } from '@aptx/frontend-tk-core';
import { functionsCommand } from './functions';
import { modelCommand } from './model';
import { toolsCommand } from './tools';

/**
 * Python codegen plugin descriptor
 */
const pythonPlugin: Plugin = {
  descriptor: {
    name: '@aptx/frontend-tk-plugin-python',
    version: '0.1.0',
    namespaceDescription: 'Python codegen (functions, model, tools)',
  },
  commands: [functionsCommand, modelCommand, toolsCommand],
  renderers: [],
};

export default pythonPlugin;
