import path from "path";

const cwd = process.cwd();

/**
 * Ensures a path is absolute, converting relative paths to absolute
 */
export function ensureAbsolutePath(p: string): string {
  if (path.isAbsolute(p)) {
    return p;
  }
  return path.join(cwd, p);
}
