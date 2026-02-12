import path from "path";
import os from "os";
import fs from "fs";
import crypto from "crypto";
import { ensureAbsolutePath } from "../../utils";

export async function getInput(input: string): Promise<string> {
  if (isUrl(input)) {
    return await downloadJsonFile(input);
  }
  return ensureAbsolutePath(input);
}

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

export function isUrl(url: string): boolean {
  return /^https?:\/\//.test(url);
}

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
