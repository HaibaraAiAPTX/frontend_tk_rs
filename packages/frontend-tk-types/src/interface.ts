export type APTXTerminalConfig = string | {
  id: string,
  output?: string,
}

export interface APTXCodegenConfig {
  outputRoot?: string,
  terminals?: APTXTerminalConfig[],
  modelOutput?: string,
}

export interface APTXPerformanceConfig {
  concurrency?: "auto" | number,
  cache?: boolean,
  format?: "fast" | "safe" | "strict",
}

export interface APTXFtConfig {
  plugin?: string[],
  input?: string,
  codegen?: APTXCodegenConfig,
  performance?: APTXPerformanceConfig,
}
