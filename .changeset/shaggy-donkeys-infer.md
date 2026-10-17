---
"@aptx-js/frontend-tk-core": patch
---

修复 OpenAPI schema 没有 `type` 时整份 spec 反序列化失败的问题。

`additionalProperties: {}` 是合法的 OpenAPI 3.0 / 3.1 写法，但 `SchemaEnum` 原先强制要求
`type`，导致 `SchemaObjectAdditionalProperties` 的 untagged 分支全部失配，整份 spec 直接报
`ERR: data did not match any variant of untagged enum SchemaObjectAdditionalProperties`。

现在 `type` **缺失**时会按结构关键字推断类型：`items` → array、`properties` /
`additionalProperties` → object、`enum` → 由第一个成员决定、其余（含 `{}`）→ 自由 object。
显式 `type` 永远优先。

`uniqueItems` / `minItems` / `maxItems` 刻意**不**参与推断：它们是约束而不是结构，单独出现时
判成 array 只会撞上 `SchemaArray` 必填的 `items`，报出 `missing field \`items\`` 这种把调用方
指向错误方向的错误，还让整份 spec 失败。

不会静默降级的形态（`SchemaTypeEnum` 是单值枚举，猜错比报错更糟）：`type` 不是字符串
（3.1 的 `type: [...]`、显式 `type: null`）、`enum` 混用多种 JSON 类型、`enum` 成员是
object / array、`enum` 为空或含 `null`、布尔 schema 带 `enum`（`SchemaBool` 没有 `enum`
字段，静默接受会让取值约束凭空消失），都会抛出带原始内容的明确错误。

同时修掉两处同层缺陷：

- 顶层无 `properties` 且不是 `additionalProperties: false` 的 schema 不再产出空的
  `export interface X {}`，改为 `export type X = object`（自由对象）。
- `type: boolean` 属性的 `description` 不再被静默丢弃，产物会带上 `/** 说明 */` 注释。

13 份真实 spec（1121 个模型 + 13 个 `manifest.json` = 每个 terminal 各 1134 个生成文件）以修复前
构建全量对拍：TypeScript 侧仅 123 个文件、181 行纯新增注释（零删除、零其它行改动），Python 侧
逐字节一致；两棵产物树均通过 `tsc --strict`，Python 产物全部通过语法解析。
