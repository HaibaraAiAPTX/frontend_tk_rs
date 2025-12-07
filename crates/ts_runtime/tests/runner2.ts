import { sleep } from './utils'

await sleep(100)

// @ts-ignore
globalThis.result = "hello world"