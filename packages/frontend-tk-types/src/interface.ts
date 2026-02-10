export type APTXTerminalConfig = string | {
  id: string,
  output?: string,
}

export interface APTXCodegenConfig {
  outputRoot?: string,
  terminals?: APTXTerminalConfig[],
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
