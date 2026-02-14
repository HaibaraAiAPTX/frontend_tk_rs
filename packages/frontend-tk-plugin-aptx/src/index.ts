/**
 * @aptx/frontend-tk-plugin-aptx
 *
 * Aptx namespace commands for the frontend toolkit CLI.
 * Provides commands like aptx:functions, aptx:react-query, aptx:vue-query.
 */

import type {
  Plugin,
  RendererDescriptor,
} from '@aptx/frontend-tk-core';
import { functionsCommand } from './functions';
import { reactQueryCommand } from './react-query';
import { vueQueryCommand } from './vue-query';

/**
 * Aptx plugin descriptor
 */
const aptxPlugin: Plugin = {
  descriptor: {
    name: '@aptx/frontend-tk-plugin-aptx',
    version: '0.1.0',
    namespaceDescription: '@aptx ecosystem (functions, react-query, vue-query)',
  },
  commands: [functionsCommand, reactQueryCommand, vueQueryCommand],
  renderers: [],
};

export default aptxPlugin;
