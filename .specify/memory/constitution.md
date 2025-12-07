<!--
SYNC IMPACT REPORT
Version: 1.0.0 -> 1.1.0
Modified Principles:
- I. Rust-Centric Core -> I. High-Performance Rust Core
- V. Test-Driven Reliability -> V. Rigorous Testing Standards
Added Principles:
- VI. Code Quality & Hygiene
- VII. Consistent User Experience
Templates requiring updates:
- .specify/templates/plan-template.md: ✅ updated (implicitly compatible)
- .specify/templates/spec-template.md: ✅ updated (implicitly compatible)
- .specify/templates/tasks-template.md: ✅ updated (implicitly compatible)
Follow-up TODOs:
- None
-->

# Frontend Toolkit RS Constitution

## Core Principles

### I. High-Performance Rust Core
The foundation of the project is built in Rust to ensure high performance, memory safety, and reliability. Critical paths must be optimized for low latency and minimal resource usage. All core logic, including the runtime and bundler, resides in Rust crates to leverage zero-cost abstractions.

### II. Flexible JS/TS Runtime
The runtime must support executing both full JavaScript/TypeScript files and individual functions within them. It leverages QuickJS (via `quickjs_runtime`) for efficient execution. This flexibility allows the toolkit to be used for diverse tasks, from running scripts to invoking specific logic.

### III. Modular Architecture
Functionality is distributed across specialized crates (e.g., `ts_runtime`, `simple_bundler`, `node_binding`). Dependencies should be minimized and explicit. Each crate should have a clear responsibility and well-defined public API.

### IV. Binding & Interop
Seamless integration with Node.js and other environments is a priority. Bindings (e.g., `node_binding`) must be maintained to allow cross-platform usage. The toolkit should be consumable from both Rust and Node.js environments.

### V. Rigorous Testing Standards
Testing is mandatory and non-negotiable. Unit tests must cover core logic with high coverage. Integration tests must verify end-to-end workflows and inter-crate contracts. Regressions must be captured by new tests before fixes are implemented.

### VI. Code Quality & Hygiene
Code must be idiomatic, readable, and maintainable. Rust code must pass `clippy` checks and be formatted with `rustfmt`. JS/TS code must adhere to standard linting rules. Public APIs must be documented. Technical debt should be actively managed and minimized.

### VII. Consistent User Experience
Interfaces (CLI, API, and Errors) must be predictable and intuitive. Error messages must be clear, actionable, and consistent across the toolkit. Output formats (e.g., JSON, text) must be standardized to enable easy composition and automation.

## Technical Constraints

- **Primary Language**: Rust (2021 edition).
- **Scripting Engine**: QuickJS (via `quickjs_runtime`).
- **Target Platforms**: Windows, Linux, macOS, Android.
- **Node.js Binding**: N-API (via `napi-rs` or similar, implied by `node_binding`).

## Development Workflow

- **Code Style**: `rustfmt` for Rust, standard Prettier/ESLint for JS/TS.
- **CI/CD**: Automated builds and tests on push.
- **Documentation**: Public APIs must be documented.

## Governance

The Constitution is the supreme source of architectural truth for Frontend Toolkit RS.
- **Amendments**: Require documentation, approval, and a version bump.
- **Compliance**: All PRs and reviews must verify compliance with these principles.
- **Versioning**: Follows Semantic Versioning. Major version bumps for breaking governance changes.

**Version**: 1.1.0 | **Ratified**: 2025-12-07 | **Last Amended**: 2025-12-07
