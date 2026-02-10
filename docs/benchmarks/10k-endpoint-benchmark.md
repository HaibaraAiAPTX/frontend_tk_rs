# 10k Endpoint Benchmark

- Generated at: 2026-02-10T01:55:56.030Z
- Machine: win32 10.0.26100
- CPU: AMD Ryzen 7 3700X 8-Core Processor             
- Node: v22.21.1
- Endpoint count: 10000
- Terminals: axios-ts

## Result

| Scenario | Duration(ms) | Planned | Written | Skipped | CacheHit |
|---|---:|---:|---:|---:|---|
| cold-run | 6909 | 20 | 20 | 0 | false |
| warm-run | 1 | 20 | 20 | 0 | true |
| dry-run | 7314 | 20 | 0 | 0 | false |

## Key Metrics

- warm/cold ratio: 0.0001x
- warm/cold percent: 0.01%

## Artifacts

- sample: `target/benchmarks/openapi-10000.json`
- config: `target/benchmarks/reports/benchmark.config.ts`
- cold report: `target/benchmarks/reports/cold-run.report.json`
- warm report: `target/benchmarks/reports/warm-run.report.json`
- dry report: `target/benchmarks/reports/dry-run.report.json`
