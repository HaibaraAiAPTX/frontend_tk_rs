---
"@aptx-js/frontend-tk-core": patch
---

修复 `aptx-ft --version` 恒输出 `0.1.0` 的问题。

版本号原本写死在 `cli.ts` 里，与 npm 上的实际版本脱节，每次发版都会漏改。
改为在运行时读取本包 `package.json` 的 `version`，并加回归测试锁住该约束。
