import { describe, it, expect } from 'vitest';
import * as fs from 'node:fs';
import * as path from 'node:path';

// cli.ts 在模块顶层 import 原生 binding，测试环境未必能加载，
// 因此这里只锁住"版本号不得写死"这条约束。
describe('CLI version', () => {
  const coreRoot = path.resolve(__dirname, '..');
  const cliSource = fs.readFileSync(path.join(coreRoot, 'src', 'cli.ts'), 'utf8');
  const pkg = JSON.parse(
    fs.readFileSync(path.join(coreRoot, 'package.json'), 'utf8'),
  ) as { version: string };

  it('version() 不接受字面量版本号', () => {
    // 写死会让 --version 与实际发布版本脱节，且每次发版都会漏改
    expect(cliSource).not.toMatch(/\.version\(\s*['"`]\d+\.\d+\.\d+/);
  });

  it('version() 由函数提供，读取 package.json', () => {
    expect(cliSource).toMatch(/\.version\(readPackageVersion\(\)\)/);
    expect(cliSource).toMatch(/package\.json/);
  });

  it('package.json 的 version 形如 semver', () => {
    expect(pkg.version).toMatch(/^\d+\.\d+\.\d+/);
  });
});
