#!/usr/bin/env node
// One-off helper: extract the acceptance-sample schemas (plus their $ref closure)
// from the parent repo's real swagger files into a vendored fixture.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const repoRoot = path.resolve(__dirname, "..");
const swaggerDir = path.resolve(repoRoot, "..", "swagger");
const outFile = path.join(repoRoot, "crates/swagger_gen/tests/fixtures/model_dictionary.json");

const specs = {
  "authority-swagger.json": ["GuidStringDictionaryResultModel"],
  "educational-swagger.json": ["StringClassScheduleListDTOListDictionaryResultModel"],
  "financial-swagger.json": ["AddPayrollRecordRequestModel"],
  "market-swagger.json": ["StringDecimalDictionaryResultModel"],
  "questionbank-swagger.json": ["PaperAnalysisDTO", "MatchingQuestionDTO"],
};

const schemas = {};
const pending = [];

for (const [file, names] of Object.entries(specs)) {
  const raw = fs.readFileSync(path.join(swaggerDir, file), "utf8");
  const doc = JSON.parse(raw);
  for (const name of names) {
    const schema = doc.components.schemas[name];
    if (!schema) throw new Error(`${file} is missing ${name}`);
    schemas[name] = schema;
    pending.push(schema);
  }
}

// Expand $ref closure so the fixture renders standalone.
while (pending.length > 0) {
  const node = pending.pop();
  const walk = (value) => {
    if (Array.isArray(value)) return value.forEach(walk);
    if (!value || typeof value !== "object") return;
    if (typeof value.$ref === "string") {
      const name = value.$ref.split("/").pop();
      if (!schemas[name]) {
        // Look the ref up in every source spec.
        for (const file of Object.keys(specs)) {
          const doc = JSON.parse(fs.readFileSync(path.join(swaggerDir, file), "utf8"));
          const found = doc.components.schemas[name];
          if (found) {
            schemas[name] = found;
            pending.push(found);
            return;
          }
        }
        throw new Error(`unresolved $ref: ${name}`);
      }
    }
    for (const key of Object.keys(value)) walk(value[key]);
  };
  walk(node);
}

const sorted = {};
for (const name of Object.keys(schemas).sort()) sorted[name] = schemas[name];

const fixture = {
  openapi: "3.1.0",
  info: {
    title: "model-dictionary-acceptance",
    version: "1.0.0",
    description:
      "Vendored subset of the real consumer OpenAPI specs covering additionalProperties dictionary rendering. Source: aptx-root/swagger/*.json",
  },
  paths: {},
  components: { schemas: sorted },
};

fs.mkdirSync(path.dirname(outFile), { recursive: true });
fs.writeFileSync(outFile, `${JSON.stringify(fixture, null, 2)}\n`, "utf8");
console.log(`wrote ${outFile} with ${Object.keys(sorted).length} schemas:`);
console.log(Object.keys(sorted).join(", "));
