# Data Model: JS Runtime Core

## Entities

### Runtime Configuration
Configuration options for the runtime execution.

| Field | Type | Description |
|-------|------|-------------|
| `entry_point` | `PathBuf` | Path to the main script file. |
| `func_name` | `Option<String>` | Name of the function to invoke (for `run_func`). |
| `args` | `Vec<JsValue>` | Arguments for the function. |

### Execution Result
The result of a script execution.

| Field | Type | Description |
|-------|------|-------------|
| `value` | `JsValueFacade` | The return value (resolved if Promise). |
| `error` | `String` | Error message if execution failed. |

## Internal Structures

### CustomModuleLoader
Implements `ScriptModuleLoader` to load files from the filesystem (or bundle cache).

- **Methods**:
  - `normalize_path`: Resolves relative paths.
  - `load_module`: Reads file content.

### SimpleBundler (External)
Used to pre-process and bundle TypeScript/JavaScript files.
