export type APTXTerminalConfig = string | {
  id: string,
  output?: string,
}

export interface APTXModelImportConfig {
  // Import type: "relative" for relative path or "package" for package name
  type: "relative" | "package";
  // Package name when type is "package", e.g., "@my-org/models"
  packagePath?: string;
  // Model directory relative path when type is "relative", default is "../models"
  relativePath?: string;
}

export interface APTXClientImportConfig {
  // Client mode: "global" for @aptx/api-client, "local" for relative path, "package" for custom package
  mode: "global" | "local" | "package";
  // Relative path when mode is "local", e.g., "../../api/myClient"
  clientPath?: string;
  // Package name when mode is "package", e.g., "@my-org/api-client"
  clientPackage?: string;
  // Optional custom import name (e.g., "getApiClient" -> "getCustomApiClient")
  importName?: string;
}

export interface APTXCodegenConfig {
  outputRoot?: string,
  terminals?: APTXTerminalConfig[],
  modelImport?: APTXModelImportConfig,
  clientImport?: APTXClientImportConfig,
}

export interface APTXPerformanceConfig {
  concurrency?: "auto" | number,
  cache?: boolean,
}

export interface APTXFtConfig {
  plugin?: string[],
  input?: string,
  codegen?: APTXCodegenConfig,
  scriptPluginPolicy?: {
    timeoutMs?: number,
    maxWriteFiles?: number,
    maxWriteBytes?: number,
    maxHeapMb?: number,
  },
  performance?: APTXPerformanceConfig,
}
