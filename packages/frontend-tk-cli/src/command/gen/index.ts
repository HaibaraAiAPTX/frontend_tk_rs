import { gen as napiGen } from "@aptx/frontend-tk-binding"
import fs from 'fs'
import { ensureAbsolutePath, errorLog } from "../../utils"
import { getInput, isUrl } from "./get-input"

interface GenOps {
  input?: string
  plugin?: string
  modelOutput?: string[]
  serviceOutput?: string[]
  serviceMode?: string[]
}

export async function gen(ops: GenOps) {
  let input: string | undefined = ops.input
  let modelOutput: string[] = ops.modelOutput || []
  let serviceOutput: string[] = ops.serviceOutput || []

  if (!input) {
    errorLog("未找到输入文件");
    return
  }

  if (!modelOutput?.length && !serviceOutput?.length) {
    errorLog("未找到需要生成的服务或模型");
    return
  }

  input = await getInput(input)
  modelOutput = modelOutput.map(ensureAbsolutePath)
  serviceOutput = serviceOutput.map(ensureAbsolutePath)

  if (!fs.existsSync(input)) {
    errorLog(`未找到输入文件: ${input}`)
    return
  }

  napiGen({
    input: input!,
    plugin: ops.plugin,
    modelOutput,
    serviceOutput,
    serviceMode: ops.serviceMode,
  })

  console.log('生成已完成');

  if (isUrl(ops.input!)) {
    fs.unlinkSync(input)
    console.log('已删除缓存文件');
  }
}