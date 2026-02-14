/**
 * @aptx/frontend-tk-plugin-materal
 *
 * Materal backend framework specific commands for frontend toolkit.
 * Provides enum enrichment commands for Materal backend/OpenAPI integration.
 */

import type { Plugin } from "@aptx/frontend-tk-core";
import { enumPlanCommand } from "./enum-plan";
import { enumApplyCommand } from "./enum-apply";
import { enumPatchCommand } from "./enum-patch";

/**
 * Materal plugin descriptor
 */
const materalPlugin: Plugin = {
  descriptor: {
    name: "@aptx/frontend-tk-plugin-materal",
    version: "0.1.0",
    namespaceDescription: 'Materal backend framework integration',
  },
  commands: [
    enumPatchCommand,
    enumPlanCommand,
    enumApplyCommand,
  ],
  renderers: [],
};

export default materalPlugin;
