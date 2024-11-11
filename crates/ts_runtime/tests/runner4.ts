import { sleep } from "./utils";

export function double_count(a: number) {
  return a * 2;
}

export async function async_double_count(a: number) {
  await sleep(100);
  return a * 2;
}
