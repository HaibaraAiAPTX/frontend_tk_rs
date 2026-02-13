/**
 * Terminal configuration - can be a string ID or object with ID and output path
 */
export type TerminalConfig = string | { id: string; output?: string };

/**
 * Client import mode and configuration
 */
export interface ClientImportConfig {
  mode: 'global' | 'local' | 'package';
  clientPath?: string;
  clientPackage?: string;
  importName?: string;
}

/**
 * Code generation configuration
 */
export interface CodegenConfig {
  outputRoot?: string;
  terminals?: TerminalConfig[];
  clientImport?: ClientImportConfig;
}

/**
 * Application configuration
 */
export interface AppConfig {
  input?: string;
  plugin?: string[];
  codegen?: CodegenConfig;
  scriptPluginPolicy?: ScriptPluginPolicy;
  performance?: PerformanceConfig;
}

/**
 * Script plugin execution policy limits
 */
export interface ScriptPluginPolicy {
  timeoutMs?: number;
  maxWriteFiles?: number;
  maxWriteBytes?: number;
  maxHeapMb?: number;
}

/**
 * Performance settings
 */
export interface PerformanceConfig {
  concurrency?: 'auto' | number;
  cache?: boolean;
}
