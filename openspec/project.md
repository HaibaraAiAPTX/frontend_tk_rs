# Project Context

## Purpose
`frontend_tk_rs` is a multi-crate Rust workspace backed by a pnpm workspace that generates frontend API clients, TypeScript typings, and UI scaffolding from Swagger/OpenAPI sources while exposing a web-like JavaScript runtime inside Rust binaries. The workstream covers bundling, runtime behavior, CLI tooling (`aptx-ft gen`), and language bindings so that Rust services can emit ready-to-use frontend artifacts for teams working on `Materal`-style APIs.

## Tech Stack
- **Rust (2021)** workspace: crates such as `node_binding`, `node_binding_plugin`, `simple_bundler`, `swagger_gen`, `ts_runtime`, `frontend_plugin_materal`, and supporting shared code under `common`.
- **Node.js + pnpm workspace**: packages like `frontend-tk-cli` (used by examples under `examples/frontend-tk-cli`) glue into the Rust binaries via the `@aptx/frontend-tk-cli` package.
- **TypeScript tooling**: CLI, examples, and tests under `packages/frontend-tk-cli` and `test-bundler` rely on `tsconfig.json` and pnpm scripts to enforce typing, while generated clients target modern TypeScript/ES modules.
- **OpenAPI/Swagger**: `openapi3_1.yaml` and code generators (Rust-based) drive the API surface translation into frontend code, with support for Swagger-first workflows.

## Project Conventions

### Code Style
- **Rust** code conforms to `cargo fmt`/`rustfmt` defaults. Crates that ship to Node also include `rustfmt.toml` for stable formatting, and `cargo clippy` is run before releases when possible.
- **JavaScript/TypeScript** code follows the pnpm workspace defaults; the CLI packages keep files under `src/`, prefer ES module syntax, and run formatting through whatever linters/pnpm scripts the package exposes.
- **Naming**: Rust identifiers use snake_case for functions and modules, PascalCase for structs/enums. Generated TypeScript uses camelCase for runtime helpers and PascalCase for interfaces/types derived from OpenAPI models.

### Architecture Patterns
- Layered architecture: low-level runtime (`ts_runtime`), bundler/resolution (`simple_bundler` + `resolver`), Node bindings (`node_binding`, `node_binding_plugin`), and CLI orchestration (`frontend-tk-cli`).
- Plugin bar: `frontend_plugin_materal` and shared `common` crates enforce the Materal API flavor, letting CLI/plugins contribute swagger-driven metadata.
- Cross-platform distribution: `node_binding/npm/` carries prebuilt `.node` binaries for every supported target; build artifacts follow the `node-gyp`/`cargo build` handshake for each OS/arch pair.
- Specifications and plans live under `openspec/`, and every significant change should funnel through a change proposal adhering to OpenSpec instructions before implementation.

### Testing Strategy
- **Rust tests**: run `cargo test` per crate; integration tests live under each crate’s `tests/` directory (e.g., `ts_runtime/tests`, `simple_bundler/tests`).
- **TypeScript tests**: use pnpm scripts inside `packages/frontend-tk-cli` or `test-bundler` when available. Example folders (like `examples/frontend-tk-cli`) can serve as smoke tests by running the CLI against sample services.
- **Manual verification**: ensure generated code compiles and works with `node_binding` by exercising the CLI, since binaries combine Rust runtime with Node glue code.

### Git Workflow
- `master` is the long-lived main branch; create topic branches per feature/fix named after the change or `openspec` change-id (`feature/02-js-runtime-core`).
- Follow OpenSpec process before touching core logic: read applicable specs, draft a change proposal under `openspec/changes/<id>/`, run `openspec validate --strict`, and only then implement.
- Commits should describe the logical units (e.g., `feat: add timer support to ts runtime`) and reference the change-id if a proposal exists.

## Domain Context
- The repo focuses on Swagger/OpenAPI-driven frontend generation, so knowledge of API modeling, HTTP client codegen, and service metadata is crucial.
- CLI `aptx-ft` in `examples/frontend-tk-cli` lets developer teams scaffold services/models/plugins from Swagger definitions, while runtime crates emulate a browser-like JavaScript environment (timers, promises, top-level await).
- Plugin ecosystem (Materal style APIs) expects handlers to describe operations through metadata consumed during generation, so understanding that conventions drives coherent output.

## Important Constraints
- Cross-platform builds require maintaining prebuilt Node addons for Windows (x64/arm64/ia32), Linux (gnu/musl/arm variants), macOS (x64/arm64), Android, and FreeBSD.
- Generated frontend artifacts must stay in sync with the `openapi3_1.yaml` master spec; breaking changes should be coordinated through OpenSpec proposals.
- Runtime compatibility must mimic browser semantics for timers and Promises to keep scripts portable between the embedded runtime and actual frontend environments.

## External Dependencies
- **Node.js (LTS)** with pnpm for workspace management and CLI packaging.
- **Rust toolchain** (stable channel matching the current workspace) for building crates, generating `.node` binaries, and running tests.
- **OpenAPI/Swagger** files (e.g., `openapi3_1.yaml`) describing the services the frontend artifacts target.
- **pnpm workspace tooling** (`pnpm install`, `pnpm --filter`) to manage JS packages and link the Rust-generated CLI.
- **OpenSpec CLI** for drafting, validating, and archiving changes (`openspec list`, `openspec validate --strict`).
