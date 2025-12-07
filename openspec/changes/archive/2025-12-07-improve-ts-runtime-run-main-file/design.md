## Context
`run_main_file` is the entry point for running JS/TS bundles inside the Rust runtime. Integration tests exercising the `test_us1_*` pattern expect the function to return a single `JsValueFacade` representing the script's meaningful output (global state, export value, or exported helper result).

## Goals / Non-Goals
- Goals:
  - Determine and return the most appropriate value from the executed module so downstream callers (and tests) can consume it directly.
  - Preserve async behavior by always awaiting pending promises returned from the module or default export.
- Non-Goals:
  - Generalize to arbitrary export selection (reserved for `run_func`).
  - Introduce a new configuration surface beyond the implicit `default` export and `globalThis.result` conventions used today.

## Decisions
1. **Return value precedence**: inspect the module namespace after execution. Prefer a `default` export, falling back to `globalThis.result` when no default export is defined or it evaluates to `undefined`.
2. **Callable defaults**: if the default export is a function, invoke it with no arguments and await its return (mirroring the implicit test expectations for exported helpers).
3. **Promise handling**: reuse existing promise-resolution helpers so the final result is always the resolved value, regardless of whether the module returned a promise, the default export returned a promise, or the invoked function returned a promise.
4. **Error surfacing**: convert any QuickJS exceptions or missing exports into descriptive strings, preserving the existing `Result<JsValueFacade, String>` contract.

## Risks / Trade-offs
- Adding a `globalThis.result` fallback coupled to tests adds implicit requirements; clarify the touchpoint in documentation to avoid surprising future contributors.
- Invoking default export functions introduces another execution step; ensure side effects stay deterministic to avoid double-running initialization logic.

## Open Questions
- Should we allow callers to override the return resolution order (default export vs. global result)? Not needed for the scoped change; revisit if future requirements surface.
