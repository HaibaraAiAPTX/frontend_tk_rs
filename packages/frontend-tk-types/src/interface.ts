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

export interface APTXCodegenConfig {
  outputRoot?: string,
  terminals?: APTXTerminalConfig[],
  modelImport?: APTXModelImportConfig,
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
