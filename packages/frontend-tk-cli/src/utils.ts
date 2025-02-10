import chalk from "chalk";
import path from "path";

const cwd = process.cwd();

/**
 * 确保是绝对路径
 * @param p 
 * @returns 
 */
export function ensureAbsolutePath(p: string) {
  if (path.isAbsolute(p)) {
    return p
  }
  return path.join(cwd, p)
}

export function errorLog(msg: string) {
  console.log(chalk.red(msg));
}