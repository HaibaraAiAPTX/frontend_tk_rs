## 1. Implementation
- [ ] Update `run_main_file` so that after bundling/executing the entry module it inspects the module namespace for a `default` export before falling back to `globalThis.result`.
- [ ] Invoke the default export when it is callable, reuse the promise-resolution flow, and return the awaited value via `JsValueFacade`.
- [ ] Ensure `run_main_file` still surfaces errors with descriptive strings when QuickJS fails to execute the module or access the return value.
- [ ] Confirm the `test_us1_*` suite in `crates/ts_runtime/tests/runner.rs` asserts against the new semantics.

## 2. Validation
- [ ] Run `cargo test -p ts_runtime -- test_us1` (or equivalent) to re-validate the targeted tests.
- [ ] Run `openspec validate improve-ts-runtime-run-main-file --strict` to ensure the proposal passes validation.
