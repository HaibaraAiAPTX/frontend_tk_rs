# Quickstart: JS Runtime Core

## Installation

Add `ts_runtime` to your `Cargo.toml`:

```toml
[dependencies]
ts_runtime = { path = "crates/ts_runtime" }
```

## Usage

### Running a Script

```rust
use std::path::PathBuf;
use ts_runtime::runner::run_main_file;

#[tokio::main]
async fn main() {
    let script_path = PathBuf::from("path/to/script.ts");
    match run_main_file(&script_path).await {
        Ok(result) => println!("Result: {:?}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Invoking a Function

```rust
use std::path::PathBuf;
use ts_runtime::runner::run_func;
use quickjs_runtime::quickjsvalueadapter::QuickJsValueAdapter;

#[tokio::main]
async fn main() {
    let script_path = PathBuf::from("path/to/lib.ts");
    let func_name = "add".to_string();
    
    let result = run_func(&script_path, func_name, || {
        // Return arguments as QuickJsValueAdapter
        vec![
            // ... construct args
        ]
    }).await;

    match result {
        Ok(val) => println!("Function returned: {:?}", val),
        Err(e) => eprintln!("Function failed: {}", e),
    }
}
```

## Features

- **TypeScript Support**: Automatically bundles and compiles TS files.
- **Async/Await**: Supports top-level await and async functions.
- **Timers**: `setTimeout` and `setInterval` are available in scripts.
- **Modules**: Supports ES modules (`import`/`export`).
