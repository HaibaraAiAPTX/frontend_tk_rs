import path from "path";

const cwd = process.cwd();

/**
 * 确保是绝对路径
 */
export function ensureAbsolutePath(p: string): string {
  if (path.isAbsolute(p)) {
    return p;
  }
  return path.join(cwd, p);
}
