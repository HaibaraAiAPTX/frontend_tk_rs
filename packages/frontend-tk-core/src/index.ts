/**
 * @aptx/frontend-tk-core
 *
 * Core infrastructure for the frontend toolkit CLI plugin system.
 * Provides plugin loading, command registration, and routing infrastructure.
 */

// Export all types
export * from './types';

// Export CLI factory and interface
export { createCli, Cli } from './cli';

// Re-export commonly used types for convenience
export type {
  Plugin,
  PluginContext,
  CommandDescriptor,
  RendererDescriptor,
  OptionDescriptor,
  PluginDescriptor,
} from './types';
