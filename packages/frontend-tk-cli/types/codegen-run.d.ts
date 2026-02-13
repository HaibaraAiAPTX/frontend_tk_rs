/**
 * codegen:run convenience command
 *
 * This is a CLI-specific convenience command that orchestrates multi-terminal code generation.
 * It combines the functionality of multiple plugins' renderers.
 */
import type { CommandDescriptor } from "@aptx/frontend-tk-core";
export declare function createCodegenRunCommand(): CommandDescriptor;
