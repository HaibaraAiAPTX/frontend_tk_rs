import { frontendTkGen } from "@aptx/frontend-tk-binding"
import fs from 'fs'
import parser from 'yargs-parser'
import { ensureAbsolutePath, errorLog } from "./utils"
import { getInput, isUrl } from "./get-input"

interface RunOps {
  input?: string
  modelOutput?: string
  serviceOutput?: string[]
  serviceMode?: string
  genServicePlugin?: string
}

export async function run(args: string[]) {
  const ops = parser(args, {
    string: ['input', 'modelOutput', 'serviceMode', 'genServicePlugin'],
    array: ['serviceOutput'],
  }) as RunOps

  let input: string | undefined = ops.input
  let modelOutput: string | undefined = ops.modelOutput
  let serviceOutput: string[] = ops.serviceOutput || []

  if (!input) {
    errorLog("未找到输入文件");
    return
  }

  if (!modelOutput && !serviceOutput.length) {
    errorLog("未找到需要生成的服务或模型");
    return
  }

  input = await getInput(input)
  if (modelOutput) {
    modelOutput = ensureAbsolutePath(modelOutput)
  }
  serviceOutput = serviceOutput.map(ensureAbsolutePath)

  if (!fs.existsSync(input)) {
    errorLog(`未找到输入文件: ${input}`)
    return
  }

  frontendTkGen({
    input: input!,
    modelOutput,
    serviceOutput,
    serviceMode: ops.serviceMode,
    genServicePlugin: ops.genServicePlugin
  })

  console.log('生成已完成');

  if (isUrl(ops.input!)) {
    fs.unlinkSync(input)
    console.log('已删除缓存文件');
  }
}