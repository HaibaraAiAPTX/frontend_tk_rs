# Change: Improve run_main_file result resolution

## Why
`run_main_file` today only evaluates the transpiled module and returns whatever `QuickJs` hands back (typically the module namespace). The `test_us1_*` regression tests in `crates/ts_runtime/tests/runner.rs` expect the helper to return a meaningful value such as `globalThis.result`, a default export value, or the return of the default export function. Without an explicit post-processing step, these scenarios cannot be verified.

## What Changes
- Make `run_main_file` post-process the executed module so it returns either an explicit default export (invoking it when it is a function) or a `globalThis.result` value when no default export is present.
- Ensure the helper still awaits any returned promises (top-level module results, exported async functions, or async helpers) before returning to the caller.
- Document the new behavior in the JS runtime spec and keep the `test_us1_*` suite green by aligning expectations with the documented semantics.

## Impact
- Affected specs: `002-js-runtime-core`
- Affected code: `crates/ts_runtime/src/runner.rs`, `crates/ts_runtime/tests/runner.rs`, any supporting helpers documenting the runtime.
