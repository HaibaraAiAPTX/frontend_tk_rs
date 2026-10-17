---
"@aptx-js/frontend-tk-core": patch
---

修复 OpenAPI schema 没有 `type` 时整份 spec 反序列化失败的问题。

`additionalProperties: {}` 是合法的 OpenAPI 3.0 / 3.1 写法，但 `SchemaEnum` 原先强制要求
`type`，导致 `SchemaObjectAdditionalProperties` 的 untagged 分支全部失配，整份 spec 直接报
`ERR: data did not match any variant of untagged enum SchemaObjectAdditionalProperties`。

现在 `type` **缺失**时会按结构关键字推断类型：`items` / `uniqueItems` / `minItems` / `maxItems`
→ array、`properties` / `additionalProperties` → object、`enum` → 按首个成员、其余（含 `{}`）
→ 自由 object。显式 `type` 永远优先。

不会静默降级的形态（`SchemaTypeEnum` 是单值枚举，猜错比报错更糟）：`type` 不是字符串
（3.1 的 `type: [...]`、显式 `type: null`）、`enum` 混用多种 JSON 类型、`enum` 成员是
object / array、`enum` 为空或含 `null`，都会抛出带原始内容的明确错误。

同时修掉两处同层缺陷：

- 顶层无 `properties` 且不是 `additionalProperties: false` 的 schema 不再产出空的
  `export interface X {}`，改为 `export type X = object`（自由对象）。
- `type: boolean` 属性的 `description` 不再被静默丢弃，产物会带上 `/** 说明 */` 注释。

13 份真实 spec（1121 个模型、1134 个生成文件）除上述布尔属性注释外产物逐字节不变，
全部产物通过 `tsc --strict` 校验。
