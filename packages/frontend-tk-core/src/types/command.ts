/**
 * Code generation run options
 */
export interface CodegenRunOptions {
  dryRun: boolean;
  profile: boolean;
  reportJson?: string;
  concurrencyOverride?: 'auto' | number;
  cacheOverride?: boolean;
  outputRoot?: string;
  terminals?: string[];
  clientMode?: 'global' | 'local' | 'package';
  clientPath?: string;
  clientPackage?: string;
  clientImportName?: string;
}

/**
 * Global CLI options
 */
export interface GlobalOptions {
  input?: string;
  plugins: string[];
}

/**
 * Command execution result
 */
export interface CommandResult {
  success: boolean;
  error?: string;
  data?: unknown;
}
