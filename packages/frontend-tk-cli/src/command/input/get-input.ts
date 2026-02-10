import path from "path";
import { ensureAbsolutePath } from "../../utils"
import os from 'os'
import fs from 'fs'
import https from 'https'
import http from 'http'

export async function getInput(input: string): Promise<string> {
  if (isUrl(input)) {
    return await downloadJsonFile(input)
  }
  return ensureAbsolutePath(input)
}

async function downloadJsonFile(url: string): Promise<string> {
  return new Promise<string>((resolve, reject) => {
    // 获取缓存文件夹路径
    const cacheDir = os.tmpdir();

    // 从 URL 中提取文件名
    const fileName = path.basename(url);

    // 拼接缓存文件路径
    const filePath = path.join(cacheDir, fileName);

    // 创建可写流
    const fileStream = fs.createWriteStream(filePath);

    const method = url.startsWith("https://") ? https : http;

    console.log('开始下载文件');

    // 下载文件
    method.get(urlToOptions(url), (response) => {
      // 检查响应状态码
      if (response.statusCode !== 200) {
        console.error(`Failed to download file: ${response.statusCode}`);
        return;
      }

      // 将响应数据写入文件
      response.pipe(fileStream);

      // 下载完成
      fileStream.on('finish', () => {
        fileStream.close();
        console.log(`File downloaded to: ${filePath}`);
        resolve(filePath)
      });
    }).on('error', (err) => {
      // 删除未完成的文件
      fs.unlink(filePath, () => { });
      reject(new Error(`Error downloading file: ${err.message}`))
    });
  })
}

export function isUrl(url: string): boolean {
  return /^https?:\/\//.test(url)
}

function urlToOptions(url: string): https.RequestOptions | http.RequestOptions {
  const urlObj = new URL(url);
  const options: https.RequestOptions | http.RequestOptions = {
    hostname: urlObj.hostname,
    port: urlObj.port ? parseInt(urlObj.port) : (urlObj.protocol === 'https:' ? 443 : 80),
    path: urlObj.pathname,
    method: 'GET',
    rejectUnauthorized: false,
    headers: {
      'Accept': 'application/json',
    },
  }
  return options
}