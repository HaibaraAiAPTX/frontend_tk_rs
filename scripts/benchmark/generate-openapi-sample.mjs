#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

function parseArgs(argv) {
  const args = {
    output: "target/benchmarks/openapi-10000.json",
    endpoints: 10000,
  };
  for (let i = 0; i < argv.length; i += 1) {
    const token = argv[i];
    if (token === "--output") {
      args.output = argv[i + 1];
      i += 1;
      continue;
    }
    if (token === "--endpoints") {
      args.endpoints = Number(argv[i + 1]);
      i += 1;
    }
  }
  if (!Number.isInteger(args.endpoints) || args.endpoints <= 0) {
    throw new Error("`--endpoints` must be a positive integer.");
  }
  return args;
}

function buildOpenApi(endpointCount) {
  const paths = {};
  for (let i = 1; i <= endpointCount; i += 1) {
    const key = `/bench/v1/resource-${i}/{id}`;
    paths[key] = {
      get: {
        tags: [`bench-group-${(i % 20) + 1}`],
        summary: `Benchmark API ${i}`,
        operationId: `benchGetResource${i}`,
        parameters: [
          {
            name: "id",
            in: "path",
            required: true,
            schema: { type: "string" },
          },
          {
            name: "page",
            in: "query",
            required: false,
            schema: { type: "integer", format: "int32" },
          },
        ],
        responses: {
          200: {
            description: "ok",
            content: {
              "application/json": {
                schema: { $ref: "#/components/schemas/BenchItem" },
              },
            },
          },
        },
      },
    };
  }

  return {
    openapi: "3.1.0",
    info: {
      title: "Aptx Benchmark 10k API",
      version: "1.0.0",
    },
    paths,
    components: {
      schemas: {
        BenchItem: {
          type: "object",
          properties: {
            id: { type: "string" },
            value: { type: "string" },
            count: { type: "integer", format: "int32" },
          },
          required: ["id", "value"],
        },
      },
    },
  };
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const output = path.resolve(args.output);
  const openapi = buildOpenApi(args.endpoints);
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, JSON.stringify(openapi, null, 2));
  console.log(
    JSON.stringify(
      {
        output,
        endpoints: args.endpoints,
      },
      null,
      2,
    ),
  );
}

main();
