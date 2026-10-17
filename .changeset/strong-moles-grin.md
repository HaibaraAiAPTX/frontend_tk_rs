---
"@aptx-js/frontend-tk-binding": patch
"@aptx-js/frontend-tk-core": patch
"@aptx-js/frontend-tk-cli": patch
"@aptx-js/frontend-tk-plugin-model": patch
"@aptx-js/frontend-tk-plugin-aptx": patch
"@aptx-js/frontend-tk-plugin-input": patch
"@aptx-js/frontend-tk-plugin-materal": patch
"@aptx-js/frontend-tk-plugin-python": patch
---

修复模型生成器把 `additionalProperties` 字典渲染成裸 `object` 的缺陷：`model:gen` 与 `model:enum-apply` 现在输出 `Record<string, V>`（值可为 `$ref` / 数组 / 可空），固定键对象内联展开为 `{ A?: T }`，顶层纯字典 schema 输出 `export type X = Record<...>`。Python `model` 命令同步输出 `dict[str, V]`。非字典 schema 产物保持不变。
