/**
 * @aptx/frontend-tk-plugin-material
 *
 * Material framework specific commands for frontend toolkit.
 * Provides enum enrichment commands for Material UI/OpenAPI integration.
 */

import type { Plugin } from "@aptx/frontend-tk-core";
import { enumPlanCommand } from "./enum-plan";
import { enumApplyCommand } from "./enum-apply";

/**
 * Material plugin descriptor
 */
const materialPlugin: Plugin = {
  descriptor: {
    name: "@aptx/frontend-tk-plugin-material",
    version: "0.1.0",
    namespaceDescription: 'Material UI integration',
  },
  commands: [
    enumPlanCommand,
    enumApplyCommand,
  ],
  renderers: [],
};

export default materialPlugin;
