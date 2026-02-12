import { loadConfig } from "c12";
import type { APTXFtConfig } from "@aptx/frontend-tk-types";

export async function getConfig(configFile = "./aptx-ft.config.ts") {
  const config = await loadConfig<APTXFtConfig>({
    cwd: process.cwd(),
    configFile
  })

  return config
}
