# Research: JS Runtime Core

## Decisions

### 1. Runtime Engine
- **Decision**: Use `quickjs_runtime` crate (v0.17.0).
- **Rationale**: Lightweight, embeddable, supports ES2020+, and has Rust bindings.
- **Features**:
  - `console`: For logging.
  - `settimeout` / `setinterval`: For timers.
  - `typescript`: For parsing TS (though we use `simple_bundler` for bundling).
  - `quickjs-ng`: For modern features.

### 2. Module Loading & Bundling
- **Decision**: Use `simple_bundler` for resolution and bundling, and `CustomModuleLoader` for runtime loading.
- **Rationale**: `simple_bundler` handles the complexity of resolving imports (including TS) into a single or few files. `CustomModuleLoader` feeds these into QuickJS.
- **Flow**:
  1. `compiler::compiler` calls `SimpleBundler` to bundle the entry point.
  2. `runner::run_main_file` gets the bundled file path.
  3. `QuickJsRuntimeBuilder` is initialized with `CustomModuleLoader`.
  4. `eval_module` executes the bundle.

### 3. Async & Event Loop
- **Decision**: Use `tokio` for the Rust async runtime and `quickjs_runtime`'s event loop integration.
- **Rationale**: `quickjs_runtime` is designed to work with `tokio`.
- **Top-level Await**: Supported by `eval_module` returning a Promise.
- **Timers**: Enabled via `QuickJsRuntimeBuilder` configuration (requires explicit `.set_interval()`, `.set_timeout()` calls if not default).

### 4. Function Invocation
- **Decision**: Use `invoke_function_by_name` for `run_func`.
- **Handling Promises**: If the function returns a Promise, we must await it in Rust.
- **Implementation**: Check if result is `JsValueFacade::JsPromise` and await it.

## Unknowns & Clarifications

- **Resolved**: `quickjs_runtime` supports the required features.
- **Resolved**: `simple_bundler` is available in the workspace.
- **Resolved**: Top-level await is supported via module evaluation returning a promise.

## Alternatives Considered

- **Deno Core**: Too heavy for this specific requirement, though `ts_runtime` has commented out references to it. We are sticking to `quickjs_runtime` as requested.
