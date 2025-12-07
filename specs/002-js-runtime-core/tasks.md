# Implementation Tasks: JS Runtime Core

**Feature Branch**: `002-js-runtime-core`
**Spec**: [spec.md](spec.md)
**Plan**: [plan.md](plan.md)

## Phase 1: Setup
*Goal: Initialize project structure and dependencies.*

- [X] T001 Verify and update `crates/ts_runtime/Cargo.toml` dependencies (`quickjs_runtime`, `simple_bundler`, `tokio`).
- [X] T002 Clean up `crates/ts_runtime/src/lib.rs` and ensure correct module exports.

## Phase 2: Foundational
*Goal: Implement core building blocks (module loading, bundling).*

- [X] T003 Implement `CustomModuleLoader` in `crates/ts_runtime/src/module_loader.rs` to load files from filesystem.
- [X] T004 Implement `compiler` function in `crates/ts_runtime/src/compiler.rs` to bundle TS/JS files using `simple_bundler`.
- [X] T005 Create runtime setup helper in `crates/ts_runtime/src/runner.rs` to encapsulate `QuickJsRuntimeBuilder` configuration.

## Phase 3: User Story 1 - Execute Script with Async Support (P1)
*Goal: Execute JS/TS scripts with Promise and top-level await support.*

- [X] T006 [US1] Implement `run_main_file` in `crates/ts_runtime/src/runner.rs` to execute bundled scripts.
- [X] T007 [US1] Implement Promise resolution logic in `run_main_file` to await returned Promises.
- [X] T008 [P] [US1] Create `crates/ts_runtime/tests/utils.ts` with helper scripts for testing.
- [X] T009 [US1] Create `crates/ts_runtime/tests/runner.rs` integration test file.
- [X] T010 [US1] Add integration test for top-level await execution in `crates/ts_runtime/tests/runner.rs`.

## Phase 4: User Story 2 - Timer Support (P1)
*Goal: Enable setTimeout and setInterval in the runtime.*

- [X] T011 [US2] Configure `QuickJsRuntimeBuilder` in `crates/ts_runtime/src/runner.rs` to enable `set_interval` and `set_timeout`.
- [X] T012 [US2] Add integration test for `setTimeout` behavior in `crates/ts_runtime/tests/runner.rs`.
- [X] T013 [US2] Add integration test for `setInterval` behavior in `crates/ts_runtime/tests/runner.rs`.

## Phase 5: User Story 3 - Invoke Specific Method (P2)
*Goal: Invoke specific exported functions from scripts.*

- [X] T014 [US3] Implement `run_func` signature in `crates/ts_runtime/src/runner.rs`.
- [X] T015 [US3] Implement argument conversion logic in `run_func` using `QuickJsValueAdapter`.
- [X] T016 [US3] Implement Promise handling in `run_func` to await async function results.
- [X] T017 [US3] Add integration test for invoking exported functions in `crates/ts_runtime/tests/runner.rs`.

## Phase 6: Polish & Cross-Cutting Concerns
*Goal: Ensure code quality, documentation, and error handling.*

- [X] T018 Add documentation comments to public APIs in `crates/ts_runtime/src/runner.rs`.
- [X] T019 Run `cargo clippy` on `crates/ts_runtime` and fix warnings.
- [X] T020 Run `cargo fmt` on `crates/ts_runtime`.

## Dependencies

1. **Setup** (T001-T002) -> **Foundational** (T003-T005)
2. **Foundational** -> **US1** (T006-T010)
3. **US1** -> **US2** (T011-T013) (Runtime setup needed)
4. **US1** -> **US3** (T014-T017) (Runtime setup needed)

## Parallel Execution Opportunities

- **Tests**: T008 (Test utils) can be done in parallel with T003-T005.
- **US2 & US3**: Once US1 is complete, US2 and US3 can be developed in parallel as they touch different parts of the runtime configuration/usage.

## Implementation Strategy

We will start by establishing the `CustomModuleLoader` and `compiler` integration (Foundational). Then we will build the `run_main_file` entry point (US1) which is the core of the runtime. Once that is working with async support, we will enable timers (US2) and add the specific function invocation capability (US3). Finally, we will polish the code and documentation.
