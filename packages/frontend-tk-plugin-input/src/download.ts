import path from "path";
import os from "os";
import fs from "fs";
import crypto from "crypto";

/**
 * Options for input:download command
 */
export interface InputDownloadOptions {
  /** OpenAPI JSON URL to download */
  url: string;
  /** Output file path */
  output: string;
}

/**
 * Download result information
 */
export interface DownloadResult {
  success: true;
  filePath: string;
  size: number;
  url: string;
}

/**
 * Get input path - handles both URLs and local file paths
 * @param input - URL or local file path
 * @returns Absolute path to the input file
 */
export async function getInput(input: string): Promise<string> {
  if (isUrl(input)) {
    return await downloadJsonFile(input);
  }
  return ensureAbsolutePath(input);
}

/**
 * Check if a string is a URL
 * @param url - String to check
 * @returns true if the string is an HTTP/HTTPS URL
 */
export function isUrl(url: string): boolean {
  return /^https?:\/\//.test(url);
}

/**
 * Ensure a path is absolute
 * @param p - Path to convert
 * @returns Absolute path
 */
function ensureAbsolutePath(p: string): string {
  if (path.isAbsolute(p)) {
    return p;
  }
  return path.join(process.cwd(), p);
}

/**
 * Download a JSON file from URL to temp directory
 * @param url - URL to download from
 * @returns Absolute path to downloaded file
 */
async function downloadJsonFile(url: string): Promise<string> {
  const timeoutMs = 30_000;
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs);

  try {
    console.log("开始下载文件");
    const response = await fetch(url, {
      signal: controller.signal,
      headers: {
        Accept: "application/json",
      },
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const contentType = response.headers.get("content-type") || "";
    if (contentType.includes("text/html")) {
      throw new Error("URL points to HTML, not OpenAPI JSON.");
    }

    const content = await response.text();
    const trimmed = content.trim();
    if (!trimmed.startsWith("{") && !trimmed.startsWith("[")) {
      throw new Error("Downloaded content is not JSON.");
    }

    // Validate JSON early, avoid passing invalid payload to parser phase later.
    JSON.parse(content);

    const cacheDir = os.tmpdir();
    const fileName = buildTempFileName(url);
    const filePath = path.join(cacheDir, fileName);
    fs.writeFileSync(filePath, content, "utf8");
    console.log(`File downloaded to: ${filePath}`);
    return filePath;
  } catch (error) {
    if (error instanceof Error && error.name === "AbortError") {
      throw new Error(`Request timeout after ${timeoutMs}ms`);
    }
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`Error downloading file: ${message}`);
  } finally {
    clearTimeout(timeout);
  }
}

/**
 * Build a unique temp file name from URL
 * @param url - Source URL
 * @returns Generated temp file name
 */
function buildTempFileName(url: string): string {
  const urlObj = new URL(url);
  const pathName = urlObj.pathname || "";
  const baseName = path.basename(pathName) || "swagger.json";
  const safeName = baseName.includes(".json") ? baseName : `${baseName}.json`;
  const hash = crypto
    .createHash("sha1")
    .update(url)
    .digest("hex")
    .slice(0, 10);
  return `aptx-${hash}-${safeName}`;
}

/**
 * Ensure directory exists for a file path
 * @param filePath - File path
 */
function ensureDirectoryForFile(filePath: string): void {
  const dir = path.dirname(filePath);
  fs.mkdirSync(dir, { recursive: true });
}

/**
 * Execute the input:download command
 * @param options - Download options
 * @returns Download result
 */
export async function runInputDownload(
  options: InputDownloadOptions,
): Promise<DownloadResult> {
  const downloaded = await getInput(options.url);
  const output = ensureAbsolutePath(options.output);
  ensureDirectoryForFile(output);
  fs.copyFileSync(downloaded, output);
  const stat = fs.statSync(output);
  return {
    success: true,
    filePath: output,
    size: stat.size,
    url: options.url,
  };
}
